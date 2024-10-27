// Copyright (C) 2023-2024 Vladislav Nepogodin
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
    use crate::hwd_misc;

    #[test]
    fn gpu_from_amdgpu_path() {
        assert_eq!(
            hwd_misc::get_sysfs_busid_from_amdgpu_path(
                "/sys/bus/pci/drivers/amdgpu/0000:c2:00.0/ip_discovery/die/0/GC/0/"
            ),
            "0000:c2:00.0"
        );
        assert_eq!(
            hwd_misc::get_sysfs_busid_from_amdgpu_path(
                "/sys/bus/pci/drivers/amdgpu/0000:c2:00.0/ip_discovery/die//"
            ),
            "0000:c2:00.0"
        );
        assert_eq!(
            hwd_misc::get_sysfs_busid_from_amdgpu_path("/sys/bus/pci/drivers/amdgpu/0000:c2:00.0/"),
            "0000:c2:00.0"
        );
        assert_eq!(
            hwd_misc::get_sysfs_busid_from_amdgpu_path(
                "/sys/bus/pci/drivers/amdgpu/0000:30:00.0/ip_discovery/die/0/GC/0"
            ),
            "0000:30:00.0"
        );
        assert_eq!(
            hwd_misc::get_sysfs_busid_from_amdgpu_path(
                "/sys/bus/pci/drivers/amdgpu/0000:30:00.0/ip_discovery/die//"
            ),
            "0000:30:00.0"
        );
        assert_eq!(
            hwd_misc::get_sysfs_busid_from_amdgpu_path("/sys/bus/pci/drivers/amdgpu/0000:30:00.0/"),
            "0000:30:00.0"
        );
        assert_eq!(
            hwd_misc::get_sysfs_busid_from_amdgpu_path(
                "/sys/bus/pci/drivers/amdgpu/0000:04:00.0/ip_discovery/die/0/GC/0"
            ),
            "0000:04:00.0"
        );
        assert_eq!(
            hwd_misc::get_sysfs_busid_from_amdgpu_path(
                "/sys/bus/pci/drivers/amdgpu/0000:04:00.0/ip_discovery/die//"
            ),
            "0000:04:00.0"
        );
        assert_eq!(
            hwd_misc::get_sysfs_busid_from_amdgpu_path("/sys/bus/pci/drivers/amdgpu/0000:04:00.0/"),
            "0000:04:00.0"
        );

        assert_eq!(hwd_misc::get_sysfs_busid_from_amdgpu_path("/sys/bus/pci/drivers/amdgpu/"), "");
    }
}
