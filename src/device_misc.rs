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

use crate::device::Device;
use crate::{console_writer, fl, profile_misc};

pub fn print_available_profiles_in_detail(devices: &[Device]) {
    let mut config_found = false;
    for device in devices {
        let available_profiles = &device.available_profiles;
        let installed_profiles = &device.installed_profiles;
        if available_profiles.is_empty() && installed_profiles.is_empty() {
            continue;
        }
        config_found = true;

        log::info!(
            "{} {}: {} ({}:{}:{})",
            "PCI",
            fl!("device"),
            device.sysfs_id,
            device.class_id,
            device.vendor_id,
            device.device_id
        );
        println!("  {} {} {}", device.class_name, device.vendor_name, device.device_name);
        println!();
        if !installed_profiles.is_empty() {
            println!("  > {}:\n", fl!("installed"));
            for installed_profile in installed_profiles {
                profile_misc::print_profile_details(installed_profile);
            }
            println!("\n");
        }
        if !available_profiles.is_empty() {
            println!("  > {}:\n", fl!("available"));
            for available_profile in available_profiles {
                profile_misc::print_profile_details(available_profile);
            }
            println!("\n");
        }
    }

    if !config_found {
        console_writer::print_warn_msg!("no-profile-device");
    }
}
