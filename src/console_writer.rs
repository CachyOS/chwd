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

use crate::data::Data;
use crate::device::Device;
use crate::misc::Message;
use crate::profile::Profile;

use colored::Colorize;

pub fn handle_arguments_listing(data: &Data, args: &crate::Args) {
    // Check for invalid profiles
    for invalid_profile in data.invalid_profiles.iter() {
        print_warning(&format!("profile '{invalid_profile}' is invalid!"));
    }

    // List all profiles
    if args.list_all && args.show_pci {
        let all_pci_profiles = &data.all_pci_profiles;
        if !all_pci_profiles.is_empty() {
            list_profiles(all_pci_profiles, "All PCI profiles:");
        } else {
            print_warning("No PCI profiles found!");
        }
    }
    if args.list_all && args.show_usb {
        let all_usb_profiles = &data.all_usb_profiles;
        if !all_usb_profiles.is_empty() {
            list_profiles(all_usb_profiles, "All USB profiles:");
        } else {
            print_warning("No USB profiles found!");
        }
    }

    // List installed profiles
    if args.list_installed && args.show_pci {
        let installed_pci_profiles = &data.installed_pci_profiles;
        if args.detail {
            print_installed_profiles("PCI", installed_pci_profiles);
        } else if !installed_pci_profiles.is_empty() {
            list_profiles(installed_pci_profiles, "Installed PCI configs:");
        } else {
            print_warning("No installed PCI configs!");
        }
    }
    if args.list_installed && args.show_usb {
        let installed_usb_profiles = &data.installed_usb_profiles;
        if args.detail {
            print_installed_profiles("USB", installed_usb_profiles);
        } else if !installed_usb_profiles.is_empty() {
            list_profiles(installed_usb_profiles, "Installed USB configs:");
        } else {
            print_warning("No installed USB configs!");
        }
    }

    // List available profiles
    if args.list_available && args.show_pci {
        let pci_devices = &data.pci_devices;
        if args.detail {
            crate::device::print_available_profiles_in_detail("PCI", pci_devices);
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

    if args.list_available && args.show_usb {
        let usb_devices = &data.usb_devices;
        if args.detail {
            crate::device::print_available_profiles_in_detail("USB", usb_devices);
        } else {
            for usb_device in usb_devices.iter() {
                let available_profiles = &usb_device.get_available_profiles();
                if available_profiles.is_empty() {
                    continue;
                }

                list_profiles(
                    available_profiles,
                    &format!(
                        "{} ({}:{}:{}) {} {}:",
                        usb_device.sysfs_busid,
                        usb_device.class_id,
                        usb_device.vendor_id,
                        usb_device.device_id,
                        usb_device.class_name,
                        usb_device.vendor_name
                    ),
                );
            }
        }
    }
}

pub fn list_devices(devices: &Vec<Device>, type_of_device: &str) {
    if devices.is_empty() {
        print_warning(&format!("No {} devices found!", type_of_device));
        return;
    }
    print_status(&format!("{} devices:", type_of_device));
    print_line();
    println!(
        "{:>30}{:>15}{:>8}{:>8}{:>8}{:>10}",
        "TYPE", "BUS", "CLASS", "VENDOR", "DEVICE", "PROFILES"
    );
    print_line();
    for device in devices.iter() {
        println!(
            "{:>30}{:>15}{:>8}{:>8}{:>8}{:>10}",
            device.class_name,
            device.sysfs_busid,
            device.class_id,
            device.vendor_id,
            device.device_id,
            device.available_profiles.len()
        );
    }
    println!("\n");
}

pub fn list_profiles(profiles: &[Profile], header_msg: &str) {
    print_status(header_msg);
    print_line();
    println!("{:>20}{:>10}{:>7}", "NAME", "NONFREE", "TYPE");
    print_line();
    for profile in profiles.iter() {
        println!("{:>20}{:>10}{:>7}", profile.name, profile.is_nonfree, profile.prof_type);
    }
    println!("\n");
}

pub fn print_installed_profiles(device_type: &str, installed_profiles: &Vec<Profile>) {
    if installed_profiles.is_empty() {
        print_warning(&format!("no installed profile for {device_type} devices found!"));
        return;
    }

    for profile in installed_profiles.iter() {
        crate::profile::print_profile_details(profile);
    }
    println!();
}

pub fn print_message(msg_type: Message, msg_str: &str) {
    match msg_type {
        Message::InstallStart => print_status(&format!("Installing {msg_str} ...")),
        Message::InstallEnd => print_status(&format!("Successfully installed {msg_str}")),
        Message::RemoveStart => print_status(&format!("Removing {msg_str} ...")),
        Message::RemoveEnd => print_status(&format!("Successfully removed {msg_str}")),
    }
}

pub fn print_line() {
    print!("{:->50}", "\n"); // use '-' as a fill char
}

pub fn print_warning(msg: &str) {
    println!("{} {}", "Warning:".yellow(), msg);
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", "Error:".red(), msg);
}

pub fn print_status(msg: &str) {
    println!("{} {}", ">".red(), msg);
}
