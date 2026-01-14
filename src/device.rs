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

use crate::profile::Profile;

use std::sync::Arc;

#[derive(Debug, Default, Clone)]
pub struct Device {
    pub class_name: String,
    pub device_name: String,
    pub vendor_name: String,
    pub class_id: String,
    pub device_id: String,
    pub vendor_id: String,
    pub sysfs_busid: String,
    pub sysfs_id: String,
    pub available_profiles: Vec<Arc<Profile>>,
    pub installed_profiles: Vec<Arc<Profile>>,
}

impl Device {
    #[must_use]
    pub fn get_available_profiles(&self) -> Vec<Profile> {
        let smth_handle_arc = |profile: &Profile| profile.clone();
        self.available_profiles.iter().map(|x| smth_handle_arc(x)).collect()
    }

    #[must_use]
    pub fn device_info(&self) -> String {
        format!(
            "{} ({}:{}:{}) {} {} {}",
            self.sysfs_busid,
            self.class_id,
            self.vendor_id,
            self.device_id,
            self.class_name,
            self.vendor_name,
            self.device_name
        )
    }
}

#[must_use]
pub fn get_unique_devices(devices: &[Device]) -> Vec<Device> {
    let mut uniq_devices = vec![];
    for device in devices {
        // Check if already in list
        let found = uniq_devices.iter().any(|x: &Device| {
            (device.sysfs_busid == x.sysfs_busid) && (device.sysfs_id == x.sysfs_id)
        });

        if !found {
            uniq_devices.push(device.clone());
        }
    }

    uniq_devices
}
