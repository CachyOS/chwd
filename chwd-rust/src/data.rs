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
use crate::ffi::{Environment, Profile};

use std::fs;
use std::path::Path;
use std::sync::Arc;

pub type ListOfProfilesT = Vec<Profile>;
pub type ListOfDevicesT = Vec<Device>;

impl Default for Environment {
    fn default() -> Environment {
        Environment {
            sync_package_manager_database: true,
            pmcache_path: crate::consts::CHWD_PM_CACHE_DIR.to_owned(),
            pmconfig_path: crate::consts::CHWD_PM_CONFIG.to_owned(),
            pmroot_path: crate::consts::CHWD_PM_ROOT.to_owned(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Data {
    pub environment: Environment,
    pub usb_devices: ListOfDevicesT,
    pub pci_devices: ListOfDevicesT,
    pub installed_usb_profiles: ListOfProfilesT,
    pub installed_pci_profiles: ListOfProfilesT,
    pub all_usb_profiles: ListOfProfilesT,
    pub all_pci_profiles: ListOfProfilesT,
    pub invalid_profiles: Vec<String>,
}

impl Data {
    pub fn new() -> Self {
        let mut res: Self = Default::default();

        res.pci_devices = fill_devices(libhd::HWItem::Pci).expect("Failed to init");
        res.usb_devices = fill_devices(libhd::HWItem::Usb).expect("Failed to init");

        res.update_profiles_data();
        res
    }

    pub fn update_installed_profile_data(&mut self) {
        // Clear profile Vec's in each device element
        for pci_device in self.pci_devices.iter_mut() {
            pci_device.installed_profiles.clear();
        }
        for usb_device in self.usb_devices.iter_mut() {
            usb_device.installed_profiles.clear();
        }

        self.installed_pci_profiles.clear();
        self.installed_usb_profiles.clear();

        // Refill data
        self.fill_installed_profiles("PCI");
        self.fill_installed_profiles("USB");

        set_matching_profiles(&mut self.pci_devices, &mut self.installed_pci_profiles, true);
        set_matching_profiles(&mut self.usb_devices, &mut self.installed_usb_profiles, true);
    }

    pub fn get_all_devices_of_profile_ffi(
        &self,
        profile: &Profile,
        found_devices: &mut Vec<Device>,
    ) {
        found_devices.clear();
        let devices = self.get_associated_devices_for_profile(profile);
        let devices = self
            .get_all_devices_of_profile(profile)
            .into_iter()
            .map(|index| devices.get(index).unwrap().clone())
            .collect::<Vec<Device>>();

        for found_device in devices.into_iter() {
            found_devices.push(found_device);
        }
    }

    pub fn get_all_devices_of_profile(&self, profile: &Profile) -> Vec<usize> {
        let devices = self.get_associated_devices_for_profile(profile);
        get_all_devices_of_profile(devices, profile)
    }

    pub fn get_associated_devices_for_profile(&self, profile: &Profile) -> &Vec<Device> {
        if "USB" == profile.prof_type {
            &self.usb_devices
        } else {
            &self.pci_devices
        }
    }

    pub fn get_invalid_profiles(&self) -> &Vec<String> {
        &self.invalid_profiles
    }

    pub fn get_all_pci_profiles(&self) -> &Vec<Profile> {
        &self.all_pci_profiles
    }

    pub fn get_all_usb_profiles(&self) -> &Vec<Profile> {
        &self.all_usb_profiles
    }

    pub fn get_installed_pci_profiles(&self) -> &Vec<Profile> {
        &self.installed_pci_profiles
    }

    pub fn get_installed_usb_profiles(&self) -> &Vec<Profile> {
        &self.installed_usb_profiles
    }

    pub fn get_pci_devices(&self) -> &Vec<Device> {
        &self.pci_devices
    }

    pub fn get_usb_devices(&self) -> &Vec<Device> {
        &self.usb_devices
    }

    pub fn get_env_mut(&mut self) -> &mut Environment {
        &mut self.environment
    }

    fn fill_installed_profiles(&mut self, profile_type: &str) {
        let db_path = if "USB" == profile_type {
            crate::consts::CHWD_USB_DATABASE_DIR
        } else {
            crate::consts::CHWD_PCI_DATABASE_DIR
        };
        let configs = if "USB" == profile_type {
            &mut self.installed_usb_profiles
        } else {
            &mut self.installed_pci_profiles
        };

        for entry in fs::read_dir(db_path).expect("Failed to read directory!") {
            let entry = entry.unwrap();
            let entry_path = entry.path();
            if Path::new(&entry_path.to_str().unwrap()).is_dir()
                || entry_path.file_name().unwrap().to_str().unwrap()
                    != crate::consts::CHWD_CONFIG_FILE
            {
                continue;
            }
            if let Ok(mut profiles) =
                crate::parse_profiles(&entry_path.as_path().to_str().unwrap(), profile_type)
            {
                configs.append(&mut profiles);
            }
            if let Ok(mut invalid_profiles) =
                crate::get_invalid_profiles(&entry_path.as_path().to_str().unwrap())
            {
                self.invalid_profiles.append(&mut invalid_profiles);
            }
        }

        configs.sort_by(|lhs, rhs| rhs.priority.cmp(&lhs.priority));
    }

    fn fill_all_profiles(&mut self, profile_type: &str) {
        let conf_path = if "USB" == profile_type {
            crate::consts::CHWD_USB_CONFIG_DIR
        } else {
            crate::consts::CHWD_PCI_CONFIG_DIR
        };
        let configs = if "USB" == profile_type {
            &mut self.all_usb_profiles
        } else {
            &mut self.all_pci_profiles
        };

        for entry in fs::read_dir(conf_path).expect("Failed to read directory!") {
            let config_file_path = format!(
                "{}/{}",
                entry.as_ref().unwrap().path().as_os_str().to_str().unwrap(),
                crate::consts::CHWD_CONFIG_FILE
            );
            if !Path::new(&config_file_path).exists() {
                continue;
            }
            if let Ok(mut profiles) = crate::parse_profiles(&config_file_path, profile_type) {
                configs.append(&mut profiles);
            }
            if let Ok(mut invalid_profiles) = crate::get_invalid_profiles(&config_file_path) {
                self.invalid_profiles.append(&mut invalid_profiles);
            }
        }

        configs.sort_by(|lhs, rhs| rhs.priority.cmp(&lhs.priority));
    }

    fn update_profiles_data(&mut self) {
        for pci_device in self.pci_devices.iter_mut() {
            pci_device.available_profiles.clear();
        }
        for usb_device in self.usb_devices.iter_mut() {
            usb_device.available_profiles.clear();
        }

        self.all_pci_profiles.clear();
        self.all_usb_profiles.clear();

        self.fill_all_profiles("PCI");
        self.fill_all_profiles("USB");

        set_matching_profiles(&mut self.pci_devices, &mut self.all_pci_profiles, false);
        set_matching_profiles(&mut self.usb_devices, &mut self.all_usb_profiles, false);

        self.update_installed_profile_data();
    }
}

fn fill_devices(item: libhd::HWItem) -> Option<ListOfDevicesT> {
    let from_hex =
        |hex_number: u16, fill: usize| -> String { format!("{:01$X}", hex_number, fill) };

    let dev_type = if item == libhd::HWItem::Usb { "USB".to_owned() } else { "PCI".to_owned() };

    // Get the hardware devices
    let mut hd_data = libhd::HDData::new();
    let hd = libhd::HD::list(&mut hd_data, item, 1, None)?;
    let mut devices = vec![];

    for iter in hd.iter_mut() {
        let item_base_class = &iter.base_class().unwrap();
        let item_sub_class = &iter.sub_class().unwrap();
        let item_vendor = &iter.vendor().unwrap();
        let item_device = &iter.device().unwrap();

        devices.push(Device {
            dev_type: dev_type.clone(),
            class_name: item_base_class.name.to_owned(),
            device_name: item_device.name.to_owned(),
            vendor_name: item_vendor.name.to_owned(),
            class_id: format!(
                "{}{}",
                from_hex(item_base_class.id as u16, 2),
                &from_hex(item_sub_class.id as u16, 2).to_lowercase()
            ),
            device_id: from_hex(item_device.id as u16, 4).to_lowercase(),
            vendor_id: from_hex(item_vendor.id as u16, 4).to_lowercase(),
            sysfs_busid: iter.sysfs_bus_id().unwrap().to_owned(),
            sysfs_id: iter.sysfs_id().unwrap().to_owned(),
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
    profiles: &mut ListOfProfilesT,
    set_as_installed: bool,
) {
    for profile in profiles.iter() {
        set_matching_profile(profile, devices, set_as_installed);
    }
}

fn get_all_devices_of_profile(devices: &ListOfDevicesT, profile: &Profile) -> Vec<usize> {
    let mut found_indices = vec![];

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
                            found = hwd_id.device_ids.iter().any(|x| {
                                x == "*" || x.to_lowercase() == i_device.device_id.to_lowercase()
                            });
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
