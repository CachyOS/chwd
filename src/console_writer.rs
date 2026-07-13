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

use crate::data::Data;
use crate::fl;
use crate::misc::Message;
use crate::profile::Profile;

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};

pub fn handle_arguments_listing(data: &Data, args: &crate::args::Args) {
    // Check for invalid profiles
    for invalid_profile in &data.invalid_profiles {
        print_warn_msg!("invalid-profile", invalid_profile = invalid_profile.as_str());
    }

    // List all profiles
    if args.list_all {
        let all_pci_profiles = &data.all_pci_profiles;
        let all_usb_profiles = &data.all_usb_profiles;
        if all_pci_profiles.is_empty() {
            print_warn_msg!("pci-profiles-not-found");
        } else {
            list_profiles(all_pci_profiles, &fl!("all-pci-profiles"));
        }
        if all_usb_profiles.is_empty() {
            print_warn_msg!("usb-profiles-not-found");
        } else {
            list_profiles(all_usb_profiles, &fl!("all-usb-profiles"));
        }
    }

    // List installed profiles
    if args.list_installed {
        let installed_profiles = data.installed_profiles();
        if args.detail {
            print_installed_profiles(&installed_profiles);
        } else if !installed_profiles.is_empty() {
            list_profiles(&installed_profiles, &fl!("installed-profiles"));
        } else {
            print_warn_msg!("no-installed-profiles");
        }
    }

    // List available profiles
    if args.list_available {
        let pci_devices = &data.pci_devices;
        let usb_devices = &data.usb_devices;
        if args.detail {
            let pci_has_profiles =
                pci_devices.iter().any(|device| !device.get_available_profiles().is_empty());
            let usb_has_profiles =
                usb_devices.iter().any(|device| !device.get_available_profiles().is_empty());

            if pci_has_profiles {
                crate::device_misc::print_available_profiles_in_detail("PCI", pci_devices);
            }
            if usb_has_profiles {
                crate::device_misc::print_available_profiles_in_detail("USB", usb_devices);
            }
            if !pci_has_profiles && !usb_has_profiles {
                print_warn_msg!("no-profile-device");
            }
        } else {
            for pci_device in pci_devices {
                let available_profiles = &pci_device.get_available_profiles();
                if available_profiles.is_empty() {
                    continue;
                }

                list_profiles(
                    available_profiles,
                    &crate::localization::terminal_text(format!(
                        "{} ({}:{}:{}) {} {}:",
                        pci_device.sysfs_busid,
                        pci_device.class_id,
                        pci_device.vendor_id,
                        pci_device.device_id,
                        pci_device.class_name,
                        pci_device.vendor_name
                    )),
                );
            }
            for usb_device in usb_devices {
                let available_profiles = &usb_device.get_available_profiles();
                if available_profiles.is_empty() {
                    continue;
                }

                list_profiles(
                    available_profiles,
                    &crate::localization::terminal_text(format!(
                        "{} ({}:{}) {} {}:",
                        usb_device.sysfs_busid,
                        usb_device.vendor_id,
                        usb_device.device_id,
                        usb_device.vendor_name,
                        usb_device.device_name
                    )),
                );
            }
        }
    }
}

pub fn list_profiles(profiles: &[Profile], header_msg: &str) {
    log::info!("{}", crate::localization::terminal_text(header_msg));
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    for profile in profiles {
        let name = crate::localization::terminal_text(&profile.name);
        let priority = crate::localization::terminal_text(profile.priority.to_string());
        if crate::localization::is_rtl() {
            table.add_row(vec![priority, name]);
        } else {
            table.add_row(vec![name, priority]);
        }
    }

    let name_header = crate::localization::terminal_text(fl!("name-header"));
    let priority_header = crate::localization::terminal_text(fl!("priority-header"));
    if crate::localization::is_rtl() {
        table.set_header(vec![priority_header, name_header]);
    } else {
        table.set_header(vec![name_header, priority_header]);
    }

    print_table(&table);
}

pub fn print_table(table: &Table) {
    if crate::localization::is_rtl() {
        for line in table.to_string().lines() {
            println!("\u{202d}{line}\u{202c}");
        }
        println!();
    } else {
        println!("{table}\n");
    }
}

pub fn print_installed_profiles(installed_profiles: &[Profile]) {
    if installed_profiles.is_empty() {
        print_warn_msg!("no-installed-profile-device");
        return;
    }

    for profile in installed_profiles {
        crate::profile_misc::print_profile_details(profile);
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
        log::error!("{}", crate::localization::terminal_text(fl!($message_id)));
    }};
    ($message_id:literal, $($args:expr),*) => {{
        log::error!("{}", crate::localization::terminal_text(fl!($message_id, $($args), *)));
    }};
}

#[macro_export]
macro_rules! print_warn_msg {
    ($message_id:literal) => {{
        log::warn!("{}", crate::localization::terminal_text(fl!($message_id)));
    }};
    ($message_id:literal, $($args:expr),*) => {{
        log::warn!("{}", crate::localization::terminal_text(fl!($message_id, $($args), *)));
    }};
}
pub(crate) use print_error_msg;
pub(crate) use print_warn_msg;
