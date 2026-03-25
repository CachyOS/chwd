// Copyright (C) 2023-2026 Vladislav Nepogodin
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

pub struct CpuInfo {
    pub vendor: String,
    pub family: String,
    pub model: String,
}

#[must_use]
pub fn get_cpu_info() -> Option<CpuInfo> {
    let content = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    parse_cpu_info(&content)
}

fn parse_cpu_info(content: &str) -> Option<CpuInfo> {
    let mut vendor = None;
    let mut family = None;
    let mut model = None;

    for line in content.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "vendor_id" if vendor.is_none() => vendor = Some(value.to_owned()),
                "cpu family" if family.is_none() => family = Some(value.to_owned()),
                "model" if model.is_none() => model = Some(value.to_owned()),
                _ => {},
            }
        }
        if vendor.is_some() && family.is_some() && model.is_some() {
            break;
        }
    }

    Some(CpuInfo { vendor: vendor?, family: family?, model: model? })
}

#[must_use]
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
        .filter_map(|path| path.to_str().map(std::borrow::ToOwned::to_owned))
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

    #[test]
    fn parse_cpu_info_intel() {
        let cpuinfo = "\
processor\t: 0
vendor_id\t: GenuineIntel
cpu family\t: 6
model\t\t: 154
model name\t: 12th Gen Intel(R) Core(TM) i7-1260P
stepping\t: 4
microcode\t: 0x432
";
        let info = super::parse_cpu_info(cpuinfo).unwrap();
        assert_eq!(info.vendor, "GenuineIntel");
        assert_eq!(info.family, "6");
        assert_eq!(info.model, "154");
    }

    #[test]
    fn parse_cpu_info_amd() {
        let cpuinfo = "\
processor\t: 0
vendor_id\t: AuthenticAMD
cpu family\t: 25
model\t\t: 80
model name\t: AMD Ryzen 7 5800X 8-Core Processor
stepping\t: 2
";
        let info = super::parse_cpu_info(cpuinfo).unwrap();
        assert_eq!(info.vendor, "AuthenticAMD");
        assert_eq!(info.family, "25");
        assert_eq!(info.model, "80");
    }

    #[test]
    fn parse_cpu_info_empty() {
        assert!(super::parse_cpu_info("").is_none());
    }
}
