// Copyright (C) 2023-2024 Vladislav Nepogodin
//
// This file is part of CachyOS chwd.
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

use std::path::Path;
use std::sync::Arc;

use crate::profile::Profile;
use crate::{consts, data};

#[derive(Debug, PartialEq)]
pub enum Transaction {
    Install,
    Remove,
}

#[derive(Debug, PartialEq)]
pub enum Status {
    Success,
    ErrorNotInstalled,
    ErrorAlreadyInstalled,
    ErrorNoMatchLocalConfig,
    ErrorScriptFailed,
    ErrorSetDatabase,
}

#[derive(Debug)]
pub enum Message {
    InstallStart,
    InstallEnd,
    RemoveStart,
    RemoveEnd,
}

#[inline]
pub fn get_current_cmdname(cmd_line: &str) -> &str {
    if let Some(trim_pos) = cmd_line.rfind('/') {
        return cmd_line.get((trim_pos + 1)..).unwrap();
    }
    cmd_line
}

pub fn find_profile(profile_name: &str, profiles: &[Profile]) -> Option<Arc<Profile>> {
    let found_profile = profiles.iter().find(|x| x.name == profile_name);
    if let Some(found_profile) = found_profile {
        return Some(Arc::new(found_profile.clone()));
    }
    None
}

pub fn check_nvidia_card() {
    let data = data::Data::new(false);
    for pci_device in data.pci_devices.iter() {
        if pci_device.available_profiles.is_empty() {
            continue;
        }

        if pci_device.vendor_id == "10de"
            && pci_device.available_profiles.iter().any(|x| x.is_nonfree)
        {
            println!("NVIDIA card found!");
            return;
        }
    }
}

pub fn check_environment() -> Vec<String> {
    let mut missing_dirs = vec![];

    if !Path::new(consts::CHWD_PCI_CONFIG_DIR).exists() {
        missing_dirs.push(consts::CHWD_PCI_CONFIG_DIR.to_owned());
    }
    if !Path::new(consts::CHWD_PCI_DATABASE_DIR).exists() {
        missing_dirs.push(consts::CHWD_PCI_DATABASE_DIR.to_owned());
    }

    missing_dirs
}

pub fn get_sysfs_busid_from_amdgpu_path(amdgpu_path: &str) -> &str {
    amdgpu_path.split('/')
        // Extract the 7th element (amdgpu id)
        .nth(6)
        .unwrap_or_default()
}

// returns Vec of ( sysfs busid, formatted GC version )
pub fn get_gc_versions() -> Option<Vec<(String, String)>> {
    use std::fs;

    let ip_match_paths = glob::glob("/sys/bus/pci/drivers/amdgpu/*/ip_discovery/die/*/GC/*/")
        .expect("Failed to read glob pattern");

    let gc_versions = ip_match_paths
        .filter_map(Result::ok)
        .filter_map(|path| path.to_str().map(|s| s.to_owned()))
        .filter_map(|ip_match_path| {
            let sysfs_busid = get_sysfs_busid_from_amdgpu_path(&ip_match_path).to_owned();

            let major =
                fs::read_to_string(format!("{ip_match_path}/major")).ok()?.trim().to_owned();
            let minor =
                fs::read_to_string(format!("{ip_match_path}/minor")).ok()?.trim().to_owned();
            let revision =
                fs::read_to_string(format!("{ip_match_path}/revision")).ok()?.trim().to_owned();

            Some((sysfs_busid, format!("{major}.{minor}.{revision}")))
        })
        .collect::<Vec<_>>();

    // Correctly check for empty Vec:
    if gc_versions.is_empty() {
        None
    } else {
        Some(gc_versions)
    }
}

#[cfg(test)]
mod tests {
    use crate::{misc, profile};

    #[test]
    fn cmdline() {
        assert_eq!(misc::get_current_cmdname("../../../testchwd"), "testchwd");
        assert_eq!(misc::get_current_cmdname("/usr/bin/testchwd"), "testchwd");
        assert_eq!(misc::get_current_cmdname("testchwd"), "testchwd");
    }

    #[test]
    fn profile_find() {
        let prof_path = "graphic_drivers-profiles-test.toml";
        let profiles = profile::parse_profiles(prof_path).expect("failed");

        assert!(misc::find_profile("nvidia-dkms", &profiles).is_some());
        assert!(misc::find_profile("nvidia-dkm", &profiles).is_none());
        assert!(misc::find_profile("nvidia-dkms.40xxcards", &profiles).is_some());
    }

    #[test]
    fn gpu_from_amdgpu_path() {
        assert_eq!(
            misc::get_sysfs_busid_from_amdgpu_path(
                "/sys/bus/pci/drivers/amdgpu/0000:c2:00.0/ip_discovery/die/0/GC/0/"
            ),
            "0000:c2:00.0"
        );
        assert_eq!(
            misc::get_sysfs_busid_from_amdgpu_path(
                "/sys/bus/pci/drivers/amdgpu/0000:c2:00.0/ip_discovery/die//"
            ),
            "0000:c2:00.0"
        );
        assert_eq!(
            misc::get_sysfs_busid_from_amdgpu_path("/sys/bus/pci/drivers/amdgpu/0000:c2:00.0/"),
            "0000:c2:00.0"
        );
        assert_eq!(
            misc::get_sysfs_busid_from_amdgpu_path(
                "/sys/bus/pci/drivers/amdgpu/0000:30:00.0/ip_discovery/die/0/GC/0"
            ),
            "0000:30:00.0"
        );
        assert_eq!(
            misc::get_sysfs_busid_from_amdgpu_path(
                "/sys/bus/pci/drivers/amdgpu/0000:30:00.0/ip_discovery/die//"
            ),
            "0000:30:00.0"
        );
        assert_eq!(
            misc::get_sysfs_busid_from_amdgpu_path("/sys/bus/pci/drivers/amdgpu/0000:30:00.0/"),
            "0000:30:00.0"
        );
        assert_eq!(
            misc::get_sysfs_busid_from_amdgpu_path(
                "/sys/bus/pci/drivers/amdgpu/0000:04:00.0/ip_discovery/die/0/GC/0"
            ),
            "0000:04:00.0"
        );
        assert_eq!(
            misc::get_sysfs_busid_from_amdgpu_path(
                "/sys/bus/pci/drivers/amdgpu/0000:04:00.0/ip_discovery/die//"
            ),
            "0000:04:00.0"
        );
        assert_eq!(
            misc::get_sysfs_busid_from_amdgpu_path("/sys/bus/pci/drivers/amdgpu/0000:04:00.0/"),
            "0000:04:00.0"
        );

        assert_eq!(misc::get_sysfs_busid_from_amdgpu_path("/sys/bus/pci/drivers/amdgpu/"), "");
    }
}
