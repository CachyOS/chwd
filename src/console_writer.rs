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

use crate::data::Data;
use crate::fl;
use crate::misc::Message;
use crate::profile::Profile;

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;

pub fn handle_arguments_listing(data: &Data, args: &crate::args::Args) {
    // Check for invalid profiles
    for invalid_profile in data.invalid_profiles.iter() {
        print_warn_msg!("invalid-profile", invalid_profile = invalid_profile.as_str());
    }

    // List all profiles
    if args.list_all {
        let all_profiles = &data.all_profiles;
        if !all_profiles.is_empty() {
            list_profiles(all_profiles, &fl!("all-pci-profiles"));
        } else {
            print_warn_msg!("pci-profiles-not-found");
        }
    }

    // List installed profiles
    if args.list_installed {
        let installed_profiles = &data.installed_profiles;
        if args.detail {
            print_installed_profiles(installed_profiles);
        } else if !installed_profiles.is_empty() {
            list_profiles(installed_profiles, &fl!("installed-pci-profiles"));
        } else {
            print_warn_msg!("no-installed-pci-profiles");
        }
    }

    // List available profiles
    if args.list_available {
        let pci_devices = &data.pci_devices;
        if args.detail {
            crate::device::print_available_profiles_in_detail(pci_devices);
        } else {
            for pci_device in pci_devices.iter() {
                let available_profiles = &pci_device.get_available_profiles();
                if available_profiles.is_empty() {
                    continue;
                }

                list_profiles(
                    available_profiles,
                    &format!(
                        "{} ({}:{}:{}) {} {}:",
                        pci_device.sysfs_busid,
                        pci_device.class_id,
                        pci_device.vendor_id,
                        pci_device.device_id,
                        pci_device.class_name,
                        pci_device.vendor_name
                    ),
                );
            }
        }
    }
}

pub fn list_profiles(profiles: &[Profile], header_msg: &str) {
    log::info!("{header_msg}");
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![&fl!("name-header"), &fl!("priority-header")]);

    for profile in profiles.iter() {
        table.add_row(vec![&profile.name, &profile.priority.to_string()]);
    }

    println!("{table}\n");
}

pub fn print_installed_profiles(installed_profiles: &[Profile]) {
    if installed_profiles.is_empty() {
        print_warn_msg!("no-installed-profile-device");
        return;
    }

    for profile in installed_profiles.iter() {
        crate::profile::print_profile_details(profile);
    }
    println!();
}

pub fn print_message(msg_type: Message, msg_str: &str) {
    match msg_type {
        Message::InstallStart => log::info!("Installing {msg_str} ..."),
        Message::InstallEnd => log::info!("Successfully installed {msg_str}"),
        Message::RemoveStart => log::info!("Removing {msg_str} ..."),
        Message::RemoveEnd => log::info!("Successfully removed {msg_str}"),
    }
}

#[macro_export]
macro_rules! print_error_msg {
    ($message_id:literal) => {{
        log::error!("{}", fl!($message_id));
    }};
    ($message_id:literal, $($args:expr),*) => {{
        log::error!("{}", fl!($message_id, $($args), *));
    }};
}

#[macro_export]
macro_rules! print_warn_msg {
    ($message_id:literal) => {{
        log::warn!("{}", fl!($message_id));
    }};
    ($message_id:literal, $($args:expr),*) => {{
        log::warn!("{}", fl!($message_id, $($args), *));
    }};
}
pub(crate) use {print_error_msg, print_warn_msg};
