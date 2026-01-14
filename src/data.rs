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

use crate::device::Device;
use crate::profile::Profile;

use std::fs;
use std::path::Path;
use std::sync::Arc;

use regex::Regex;

pub type ListOfProfilesT = Vec<Profile>;
pub type ListOfDevicesT = Vec<Device>;

#[derive(Debug, Default)]
pub struct Data {
    pub sync_package_manager_database: bool,
    pub is_ai_sdk_target: bool,
    pub pci_devices: ListOfDevicesT,
    pub installed_profiles: ListOfProfilesT,
    pub all_profiles: ListOfProfilesT,
    pub invalid_profiles: Vec<String>,
}

impl Data {
    #[must_use]
    pub fn new(is_ai_sdk: bool) -> Self {
        let mut res = Self {
            pci_devices: fill_devices().expect("Failed to init"),
            sync_package_manager_database: true,
            is_ai_sdk_target: is_ai_sdk,
            ..Default::default()
        };

        res.update_profiles_data();
        res
    }

    pub fn update_installed_profile_data(&mut self) {
        // Clear profile Vec's in each device element
        for pci_device in &mut self.pci_devices {
            pci_device.installed_profiles.clear();
        }

        self.installed_profiles.clear();

        // Refill data
        self.fill_installed_profiles();

        set_matching_profiles(&mut self.pci_devices, &self.installed_profiles, true);
    }

    fn fill_installed_profiles(&mut self) {
        let conf_path = crate::consts::CHWD_PCI_DATABASE_DIR;
        let configs = &mut self.installed_profiles;

        fill_profiles(configs, &mut self.invalid_profiles, conf_path, self.is_ai_sdk_target);
    }

    fn fill_all_profiles(&mut self) {
        let conf_path = crate::consts::CHWD_PCI_CONFIG_DIR;
        let configs = &mut self.all_profiles;

        fill_profiles(configs, &mut self.invalid_profiles, conf_path, self.is_ai_sdk_target);
    }

    fn update_profiles_data(&mut self) {
        for pci_device in &mut self.pci_devices {
            pci_device.available_profiles.clear();
        }

        self.all_profiles.clear();

        self.fill_all_profiles();

        set_matching_profiles(&mut self.pci_devices, &self.all_profiles, false);

        self.update_installed_profile_data();
    }
}

fn fill_profiles(
    configs: &mut ListOfProfilesT,
    invalid_profiles: &mut Vec<String>,
    conf_path: &str,
    is_ai_sdk: bool,
) {
    for entry in fs::read_dir(conf_path).expect("Failed to read directory!") {
        let config_file_path = format!(
            "{}/{}",
            entry.as_ref().unwrap().path().as_os_str().to_str().unwrap(),
            crate::consts::CHWD_CONFIG_FILE
        );
        if !Path::new(&config_file_path).exists() {
            continue;
        }
        let profiles = crate::profile::parse_profiles(&config_file_path)
            .expect("Urgent invalid profiles detected!");
        for profile in profiles {
            if profile.packages.is_empty() {
                continue;
            }
            // if we dont target ai sdk,
            // skip profile marked as ai sdk.
            if !is_ai_sdk && profile.is_ai_sdk {
                continue;
            }
            // if we target ai sdk,
            // skip profile which isn't marked as ai sdk.
            if is_ai_sdk && !profile.is_ai_sdk {
                continue;
            }
            configs.push(profile);
        }
        if let Ok(mut invalid_profile_names) =
            crate::profile::get_invalid_profiles(&config_file_path)
        {
            invalid_profiles.append(&mut invalid_profile_names);
        }
    }

    configs.sort_by(|lhs, rhs| rhs.priority.cmp(&lhs.priority));
}

fn fill_devices() -> Option<ListOfDevicesT> {
    #[allow(clippy::uninlined_format_args)]
    let from_hex =
        |hex_number: u32, fill: usize| -> String { format!("{:01$x}", hex_number, fill) };

    // Initialize
    let mut pacc = libpci::PCIAccess::new(true);

    // Get hardware devices
    let pci_devices = pacc.devices()?;
    let mut devices = vec![];

    for mut iter in pci_devices.iter_mut() {
        // fill in header info we need
        iter.fill_info(libpci::Fill::IDENT as u32 | libpci::Fill::CLASS as u32);

        // let item_base_class = &iter.base_class().unwrap();
        // let item_sub_class = &iter.sub_class().unwrap();
        let item_class = iter.class()?;
        let item_vendor = iter.vendor()?;
        let item_device = iter.device()?;

        devices.push(Device {
            class_name: item_class,
            device_name: item_device,
            vendor_name: item_vendor,
            class_id: from_hex(iter.class_id()?.into(), 4),
            device_id: from_hex(iter.device_id()?.into(), 4),
            vendor_id: from_hex(iter.vendor_id()?.into(), 4),
            sysfs_busid: format!(
                "{}:{}:{}.{}",
                from_hex(iter.domain()? as _, 4),
                from_hex(iter.bus()?.into(), 2),
                from_hex(iter.dev()?.into(), 2),
                iter.func()?,
            ),
            sysfs_id: String::new(),
            available_profiles: vec![],
            installed_profiles: vec![],
        });
    }

    Some(devices)
}

fn set_matching_profile(profile: &Profile, devices: &mut ListOfDevicesT, set_as_installed: bool) {
    let found_indices: Vec<usize> = get_all_devices_of_profile(devices, profile);

    // Set config to all matching devices
    for found_index in found_indices {
        let found_device = devices.get_mut(found_index).unwrap();
        let to_be_added = if set_as_installed {
            &mut found_device.installed_profiles
        } else {
            &mut found_device.available_profiles
        };
        add_profile_sorted(to_be_added, profile);
    }
}

fn set_matching_profiles(
    devices: &mut ListOfDevicesT,
    profiles: &ListOfProfilesT,
    set_as_installed: bool,
) {
    for profile in profiles {
        set_matching_profile(profile, devices, set_as_installed);
    }
}

fn get_all_devices_from_gc_versions(
    devices: &ListOfDevicesT,
    hwd_gc_versions: &[(String, String)],
    profile_gc_versions: &[String],
) -> Vec<usize> {
    let mut found_indices = vec![];
    for (sysfs_busid, gc_version) in hwd_gc_versions {
        if !profile_gc_versions.iter().any(|x| x == gc_version) {
            continue;
        }
        if let Some(device_index) =
            devices.iter().position(|device| &device.sysfs_busid == sysfs_busid)
        {
            found_indices.push(device_index);
        }
    }
    found_indices
}

#[must_use]
pub fn get_all_devices_of_profile(devices: &ListOfDevicesT, profile: &Profile) -> Vec<usize> {
    let mut found_indices = vec![];

    let dev_name_re: Option<Regex> = profile
        .device_name_pattern
        .as_ref()
        .map(|dev_pattern| Regex::new(dev_pattern).expect("Failed to initialize regex"));

    let product_name_re: Option<Regex> = profile
        .hwd_product_name_pattern
        .as_ref()
        .map(|product_pattern| Regex::new(product_pattern).expect("Failed to initialize regex"));

    if let Some(product_name_re) = &product_name_re {
        let product_name = fs::read_to_string("/sys/devices/virtual/dmi/id/product_name")
            .expect("Failed to read product name");
        if !product_name_re.is_match(&product_name) {
            return vec![];
        }
    }

    if let Some(gc_versions) = &profile.gc_versions {
        if let Some(hwd_gc_versions) = crate::hwd_misc::get_gc_versions() {
            return get_all_devices_from_gc_versions(devices, &hwd_gc_versions, gc_versions);
        }
        return vec![];
    }

    for hwd_id in &profile.hwd_ids {
        let mut found_device = false;

        // Check all devices
        for i_device_index in 0..devices.len() {
            let i_device: &Device = devices.get(i_device_index).unwrap();
            // Check class ids
            let mut found = hwd_id
                .class_ids
                .iter()
                .any(|x| x == "*" || x.to_lowercase() == i_device.class_id.to_lowercase());
            if found {
                // Check blacklisted class ids
                found = hwd_id
                    .blacklisted_class_ids
                    .iter()
                    .any(|x| x.to_lowercase() == i_device.class_id.to_lowercase());
                if !found {
                    // Check vendor ids
                    found = hwd_id
                        .vendor_ids
                        .iter()
                        .any(|x| x == "*" || x.to_lowercase() == i_device.vendor_id.to_lowercase());
                    if found {
                        // Check blacklisted vendor ids
                        found = hwd_id
                            .blacklisted_vendor_ids
                            .iter()
                            .any(|x| x.to_lowercase() == i_device.vendor_id.to_lowercase());
                        if !found {
                            // Check device ids
                            found = if let Some(dev_name_re) = &dev_name_re {
                                dev_name_re.is_match(&i_device.device_name)
                            } else {
                                hwd_id.device_ids.iter().any(|x| {
                                    x == "*"
                                        || x.to_lowercase() == i_device.device_id.to_lowercase()
                                })
                            };
                            if found {
                                // Check blacklisted device ids
                                found = hwd_id
                                    .blacklisted_device_ids
                                    .iter()
                                    .any(|x| x.to_lowercase() == i_device.device_id.to_lowercase());
                                if !found {
                                    found_device = true;
                                    found_indices.push(i_device_index);
                                }
                            }
                        }
                    }
                }
            }
        }

        if !found_device {
            found_indices.clear();
            return found_indices;
        }
    }

    found_indices
}

fn add_profile_sorted(profiles: &mut Vec<Arc<Profile>>, new_profile: &Profile) {
    if profiles.iter().any(|x| new_profile.name == x.name) {
        return;
    }

    profiles.push(Arc::new(new_profile.clone()));
    profiles.sort_by(|lhs, rhs| rhs.priority.cmp(&lhs.priority));
}

#[cfg(test)]
mod tests {
    use crate::data;
    use crate::device::Device;

    fn test_data() -> Vec<Device> {
        vec![
            Device {
                class_name: "Host bridge".to_string(),
                device_name: "Cezanne Data Fabric; Function 5".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0600".to_string(),
                device_id: "166f".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:18.5".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Audio device".to_string(),
                device_name: "Family 17h/19h HD Audio Controller".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0403".to_string(),
                device_id: "15e3".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:08:00.6".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "SMBus".to_string(),
                device_name: "FCH SMBus Controller".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0c05".to_string(),
                device_id: "790b".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:14.0".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Host bridge".to_string(),
                device_name: "Cezanne Data Fabric; Function 7".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0600".to_string(),
                device_id: "1671".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:18.7".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Host bridge".to_string(),
                device_name: "Renoir PCIe Dummy Host Bridge".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0600".to_string(),
                device_id: "1632".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:02.0".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Ethernet controller".to_string(),
                device_name: "RTL8111/8168/8211/8411 PCI Express Gigabit Ethernet Controller"
                    .to_string(),
                vendor_name: "Realtek Semiconductor Co., Ltd.".to_string(),
                class_id: "0200".to_string(),
                device_id: "8168".to_string(),
                vendor_id: "10ec".to_string(),
                sysfs_busid: "0000:04:00.0".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "PCI bridge".to_string(),
                device_name: "Renoir/Cezanne PCIe GPP Bridge".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0604".to_string(),
                device_id: "1634".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:02.2".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Host bridge".to_string(),
                device_name: "Cezanne Data Fabric; Function 0".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0600".to_string(),
                device_id: "166a".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:18.0".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Network controller".to_string(),
                device_name: "RTL8852AE 802.11ax PCIe Wireless Network Adapter".to_string(),
                vendor_name: "Realtek Semiconductor Co., Ltd.".to_string(),
                class_id: "0280".to_string(),
                device_id: "8852".to_string(),
                vendor_id: "10ec".to_string(),
                sysfs_busid: "0000:05:00.0".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Audio device".to_string(),
                device_name: "Renoir Radeon High Definition Audio Controller".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD/ATI]".to_string(),
                class_id: "0403".to_string(),
                device_id: "1637".to_string(),
                vendor_id: "1002".to_string(),
                sysfs_busid: "0000:08:00.1".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "PCI bridge".to_string(),
                device_name: "Renoir PCIe GPP Bridge".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0604".to_string(),
                device_id: "1633".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:01.1".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Host bridge".to_string(),
                device_name: "Cezanne Data Fabric; Function 2".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0600".to_string(),
                device_id: "166c".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:18.2".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "USB controller".to_string(),
                device_name: "Renoir/Cezanne USB 3.1".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0c03".to_string(),
                device_id: "1639".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:08:00.3".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "SD Host controller".to_string(),
                device_name: "GL9750 SD Host Controller".to_string(),
                vendor_name: "Genesys Logic, Inc".to_string(),
                class_id: "0805".to_string(),
                device_id: "9750".to_string(),
                vendor_id: "17a0".to_string(),
                sysfs_busid: "0000:06:00.0".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Host bridge".to_string(),
                device_name: "Cezanne Data Fabric; Function 4".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0600".to_string(),
                device_id: "166e".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:18.4".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "PCI bridge".to_string(),
                device_name: "Renoir Internal PCIe GPP Bridge to Bus".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0604".to_string(),
                device_id: "1635".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:08.1".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Multimedia controller".to_string(),
                device_name: "ACP/ACP3X/ACP6x Audio Coprocessor".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0480".to_string(),
                device_id: "15e2".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:08:00.5".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Host bridge".to_string(),
                device_name: "Renoir/Cezanne Root Complex".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0600".to_string(),
                device_id: "1630".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:00.0".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Audio device".to_string(),
                device_name: "Navi 10 HDMI Audio".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD/ATI]".to_string(),
                class_id: "0403".to_string(),
                device_id: "ab38".to_string(),
                vendor_id: "1002".to_string(),
                sysfs_busid: "0000:03:00.1".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Host bridge".to_string(),
                device_name: "Cezanne Data Fabric; Function 6".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0600".to_string(),
                device_id: "1670".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:18.6".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "IOMMU".to_string(),
                device_name: "Renoir/Cezanne IOMMU".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0806".to_string(),
                device_id: "1631".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:00.2".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Non-Volatile memory controller".to_string(),
                device_name: "3400 NVMe SSD [Hendrix]".to_string(),
                vendor_name: "Micron Technology Inc".to_string(),
                class_id: "0108".to_string(),
                device_id: "5407".to_string(),
                vendor_id: "1344".to_string(),
                sysfs_busid: "0000:07:00.0".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "PCI bridge".to_string(),
                device_name: "Renoir/Cezanne PCIe GPP Bridge".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0604".to_string(),
                device_id: "1634".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:02.1".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "PCI bridge".to_string(),
                device_name: "Navi 10 XL Upstream Port of PCI Express Switch".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD/ATI]".to_string(),
                class_id: "0604".to_string(),
                device_id: "1478".to_string(),
                vendor_id: "1002".to_string(),
                sysfs_busid: "0000:01:00.0".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "ISA bridge".to_string(),
                device_name: "FCH LPC Bridge".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0601".to_string(),
                device_id: "790e".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:14.3".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "PCI bridge".to_string(),
                device_name: "Renoir/Cezanne PCIe GPP Bridge".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0604".to_string(),
                device_id: "1634".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:02.3".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "VGA compatible controller".to_string(),
                device_name: "Cezanne [Radeon Vega Series / Radeon Vega Mobile Series]".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD/ATI]".to_string(),
                class_id: "0300".to_string(),
                device_id: "1638".to_string(),
                vendor_id: "1002".to_string(),
                sysfs_busid: "0000:08:00.0".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Host bridge".to_string(),
                device_name: "Renoir PCIe Dummy Host Bridge".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0600".to_string(),
                device_id: "1632".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:01.0".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "PCI bridge".to_string(),
                device_name: "Navi 10 XL Downstream Port of PCI Express Switch".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD/ATI]".to_string(),
                class_id: "0604".to_string(),
                device_id: "1479".to_string(),
                vendor_id: "1002".to_string(),
                sysfs_busid: "0000:02:00.0".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Host bridge".to_string(),
                device_name: "Cezanne Data Fabric; Function 1".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0600".to_string(),
                device_id: "166b".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:18.1".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Encryption controller".to_string(),
                device_name: "Family 17h (Models 10h-1fh) Platform Security Processor".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "1080".to_string(),
                device_id: "15df".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:08:00.2".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "PCI bridge".to_string(),
                device_name: "Renoir/Cezanne PCIe GPP Bridge".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0604".to_string(),
                device_id: "1634".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:01.2".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Host bridge".to_string(),
                device_name: "Cezanne Data Fabric; Function 3".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0600".to_string(),
                device_id: "166d".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:18.3".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Host bridge".to_string(),
                device_name: "Renoir PCIe Dummy Host Bridge".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0600".to_string(),
                device_id: "1632".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:00:08.0".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "USB controller".to_string(),
                device_name: "Renoir/Cezanne USB 3.1".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD]".to_string(),
                class_id: "0c03".to_string(),
                device_id: "1639".to_string(),
                vendor_id: "1022".to_string(),
                sysfs_busid: "0000:08:00.4".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
            Device {
                class_name: "Display controller".to_string(),
                device_name: "Navi 14 [Radeon RX 5500/5500M / Pro 5500M]".to_string(),
                vendor_name: "Advanced Micro Devices, Inc. [AMD/ATI]".to_string(),
                class_id: "0380".to_string(),
                device_id: "7340".to_string(),
                vendor_id: "1002".to_string(),
                sysfs_busid: "0000:03:00.0".to_string(),
                sysfs_id: "".to_string(),
                available_profiles: vec![],
                installed_profiles: vec![],
            },
        ]
    }

    #[test]
    fn get_devices_from_gc_versions() {
        let devices = test_data();
        let hwd_gc_versions = vec![
            ("0000:03:00.0".to_owned(), "10.1.1".to_owned()),
            ("0000:08:00.0".to_owned(), "9.3.0".to_owned()),
        ];
        let profile_gc_versions =
            vec!["10.1.1".to_owned(), "9.3.0".to_owned(), "10.3.1".to_owned(), "11.0.0".to_owned()];

        assert_eq!(
            data::get_all_devices_from_gc_versions(
                &devices,
                &hwd_gc_versions,
                &profile_gc_versions
            ),
            vec![35, 26]
        );
    }
}
