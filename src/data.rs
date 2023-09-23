// Copyright (C) 2023 Vladislav Nepogodin
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
    pub pci_devices: ListOfDevicesT,
    pub installed_pci_profiles: ListOfProfilesT,
    pub all_pci_profiles: ListOfProfilesT,
    pub invalid_profiles: Vec<String>,
}

impl Data {
    pub fn new() -> Self {
        let mut res = Self {
            pci_devices: fill_devices().expect("Failed to init"),
            sync_package_manager_database: true,
            ..Default::default()
        };

        res.update_profiles_data();
        res
    }

    pub fn update_installed_profile_data(&mut self) {
        // Clear profile Vec's in each device element
        for pci_device in self.pci_devices.iter_mut() {
            pci_device.installed_profiles.clear();
        }

        self.installed_pci_profiles.clear();

        // Refill data
        self.fill_installed_profiles();

        set_matching_profiles(&mut self.pci_devices, &self.installed_pci_profiles, true);
    }

    fn fill_installed_profiles(&mut self) {
        let conf_path = crate::consts::CHWD_PCI_DATABASE_DIR;
        let configs = &mut self.installed_pci_profiles;

        fill_profiles(configs, &mut self.invalid_profiles, conf_path);
    }

    fn fill_all_profiles(&mut self) {
        let conf_path = crate::consts::CHWD_PCI_CONFIG_DIR;
        let configs = &mut self.all_pci_profiles;

        fill_profiles(configs, &mut self.invalid_profiles, conf_path);
    }

    fn update_profiles_data(&mut self) {
        for pci_device in self.pci_devices.iter_mut() {
            pci_device.available_profiles.clear();
        }

        self.all_pci_profiles.clear();

        self.fill_all_profiles();

        set_matching_profiles(&mut self.pci_devices, &self.all_pci_profiles, false);

        self.update_installed_profile_data();
    }
}

fn fill_profiles(
    configs: &mut ListOfProfilesT,
    invalid_profiles: &mut Vec<String>,
    conf_path: &str,
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
        if let Ok(profiles) = crate::profile::parse_profiles(&config_file_path) {
            for profile in profiles.into_iter() {
                if profile.packages.is_empty() {
                    continue;
                }
                configs.push(profile);
            }
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
    let from_hex =
        |hex_number: u32, fill: usize| -> String { format!("{:01$x}", hex_number, fill) };

    let dev_type = "PCI".to_owned();

    // Initialize
    let mut pacc = libpci::PCIAccess::new(true);

    // Get hardware devices
    let pci_devices = pacc.devices().expect("Failed");
    let mut devices = vec![];

    for mut iter in pci_devices.iter_mut() {
        // fill in header info we need
        iter.fill_info(libpci::Fill::IDENT as u32 | libpci::Fill::CLASS as u32);

        // let item_base_class = &iter.base_class().unwrap();
        // let item_sub_class = &iter.sub_class().unwrap();
        let item_class = iter.class().unwrap();
        let item_vendor = iter.vendor().unwrap();
        let item_device = iter.device().unwrap();

        devices.push(Device {
            dev_type: dev_type.clone(),
            class_name: item_class,
            device_name: item_device,
            vendor_name: item_vendor,
            class_id: from_hex(iter.class_id().unwrap() as _, 4).to_string(),
            device_id: from_hex(iter.device_id().unwrap() as _, 4).to_string(),
            vendor_id: from_hex(iter.vendor_id().unwrap() as _, 4).to_string(),
            sysfs_busid: format!(
                "{}:{}:{}.{}",
                from_hex(iter.domain().unwrap() as _, 4),
                from_hex(iter.bus().unwrap() as _, 2),
                from_hex(iter.dev().unwrap() as _, 2),
                iter.func().unwrap(),
            ),
            sysfs_id: "".to_owned(),
            available_profiles: vec![],
            installed_profiles: vec![],
        });
    }

    Some(devices)
}

fn set_matching_profile(profile: &Profile, devices: &mut ListOfDevicesT, set_as_installed: bool) {
    let found_indices: Vec<usize> = get_all_devices_of_profile(devices, profile);

    // Set config to all matching devices
    for found_index in found_indices.into_iter() {
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
    for profile in profiles.iter() {
        set_matching_profile(profile, devices, set_as_installed);
    }
}

pub fn get_all_devices_of_profile(devices: &ListOfDevicesT, profile: &Profile) -> Vec<usize> {
    let mut found_indices = vec![];

    let re: Option<Regex> = profile
        .device_name_pattern
        .as_ref()
        .map(|dev_pattern| Regex::new(dev_pattern).expect("Failed to initialize regex"));

    for hwd_id in profile.hwd_ids.iter() {
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
                            found = if let Some(re) = &re {
                                re.is_match(&i_device.device_name)
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
    let found = profiles.iter().any(|x| new_profile.name == x.name);
    if found {
        return;
    }

    profiles.push(Arc::new(new_profile.clone()));
    profiles.sort_by(|lhs, rhs| rhs.priority.cmp(&lhs.priority));
}
