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

#![feature(extern_types)]

pub mod console_writer;
pub mod consts;
pub mod data;
pub mod device;
pub mod profile;

use console_writer::*;
use device::*;
use profile::*;

use std::path::Path;

use subprocess::Exec;

type DataFFi = data::Data;
type DeviceFFi = device::Device;

#[cxx::bridge(namespace = "chwd")]
mod ffi {
    #[derive(Debug, Default, Clone)]
    struct HardwareID {
        pub class_ids: Vec<String>,
        pub vendor_ids: Vec<String>,
        pub device_ids: Vec<String>,
        pub blacklisted_class_ids: Vec<String>,
        pub blacklisted_vendor_ids: Vec<String>,
        pub blacklisted_device_ids: Vec<String>,
    }

    #[derive(Debug, Clone)]
    pub struct Profile {
        pub is_nonfree: bool,

        pub prof_path: String,
        pub prof_type: String,
        pub name: String,
        pub desc: String,
        pub priority: i32,
        pub packages: String,

        pub hwd_ids: Vec<HardwareID>,
    }

    #[derive(Debug, Clone)]
    pub struct Arguments {
        pub show_pci: bool,
        pub show_usb: bool,
        pub install: bool,
        pub remove: bool,
        pub detail: bool,
        pub force: bool,
        pub list_all: bool,
        pub list_installed: bool,
        pub list_available: bool,
        pub list_hardware: bool,
        pub autoconfigure: bool,
    }

    #[derive(Debug)]
    pub struct Environment {
        pub sync_package_manager_database: bool,
        pub pmcache_path: String,
        pub pmconfig_path: String,
        pub pmroot_path: String,
    }

    #[derive(Debug)]
    pub enum Transaction {
        Install,
        Remove,
    }

    #[derive(Debug)]
    pub enum Status {
        Success,
        ErrorNotInstalled,
        ErrorAlreadyInstalled,
        ErrorNoMatchLocalConfig,
        ErrorScriptFailed,
        ErrorSetDatabase,
    }

    #[derive(Debug)]
    pub enum Message {
        InstallStart,
        InstallEnd,
        RemoveStart,
        RemoveEnd,
    }

    extern "Rust" {
        type DataFFi;

        fn initialize_data_obj() -> Box<DataFFi>;
        fn get_all_pci_profiles(self: &DataFFi) -> &Vec<Profile>;
        fn get_all_usb_profiles(self: &DataFFi) -> &Vec<Profile>;
        fn get_installed_pci_profiles(self: &DataFFi) -> &Vec<Profile>;
        fn get_installed_usb_profiles(self: &DataFFi) -> &Vec<Profile>;
        fn get_pci_devices(self: &DataFFi) -> &Vec<DeviceFFi>;
        fn get_usb_devices(self: &DataFFi) -> &Vec<DeviceFFi>;
        fn get_env_mut(self: &mut DataFFi) -> &mut Environment;

        fn update_installed_profile_data(self: &mut DataFFi);

        fn write_profile_to_file(file_path: &str, profile: &Profile) -> bool;
    }

    extern "Rust" {
        type DeviceFFi;

        fn get_available_profiles(self: &DeviceFFi) -> Vec<Profile>;
    }

    extern "Rust" {
        fn run_script(data: &mut Box<DataFFi>, profile: &Profile, transaction: Transaction)
            -> bool;
        fn check_nvidia_card();
        fn check_environment() -> Vec<String>;
        fn prepare_autoconfigure(
            data: &Box<DataFFi>,
            args: &mut Arguments,
            operation: &str,
            autoconf_class_id: &str,
            autoconf_nonfree_driver: bool,
        ) -> Vec<String>;

        fn handle_arguments_listing(data: &Box<DataFFi>, args: Arguments);

        fn print_message(msg_type: Message, msg_str: &str);
        fn print_warning(msg: &str);
        fn print_error(msg: &str);
        fn print_status(msg: &str);
    }
}

pub fn initialize_data_obj() -> Box<DataFFi> {
    Box::new(data::Data::new())
}

pub fn check_nvidia_card() {
    if !Path::new("/var/lib/mhwd/ids/pci/nvidia.ids").exists() {
        println!("No nvidia ids found!");
        return;
    }

    let data = data::Data::new();
    for pci_device in data.pci_devices.iter() {
        if pci_device.available_profiles.is_empty() {
            continue;
        }

        if pci_device.vendor_id == "10de" {
            println!("NVIDIA card found!");
            return;
        }
    }
}

pub fn check_environment() -> Vec<String> {
    let mut missing_dirs = vec![];

    if !Path::new(consts::CHWD_USB_CONFIG_DIR).exists() {
        missing_dirs.push(consts::CHWD_USB_CONFIG_DIR.to_owned());
    }
    if !Path::new(consts::CHWD_PCI_CONFIG_DIR).exists() {
        missing_dirs.push(consts::CHWD_PCI_CONFIG_DIR.to_owned());
    }
    if !Path::new(consts::CHWD_USB_DATABASE_DIR).exists() {
        missing_dirs.push(consts::CHWD_USB_DATABASE_DIR.to_owned());
    }
    if !Path::new(consts::CHWD_PCI_DATABASE_DIR).exists() {
        missing_dirs.push(consts::CHWD_PCI_DATABASE_DIR.to_owned());
    }

    missing_dirs
}

pub fn run_script(
    data: &mut Box<data::Data>,
    profile: &ffi::Profile,
    transaction: ffi::Transaction,
) -> bool {
    let mut cmd = format!("exec {}", consts::CHWD_SCRIPT_PATH);

    if ffi::Transaction::Remove == transaction {
        cmd.push_str(" --remove");
    } else {
        cmd.push_str(" --install");
    }

    let environment = &data.environment;
    if environment.sync_package_manager_database {
        cmd.push_str(" --sync");
    }

    cmd.push_str(&format!(" --cachedir \"{}\"", environment.pmcache_path));
    cmd.push_str(&format!(" --pmconfig \"{}\"", environment.pmconfig_path));
    cmd.push_str(&format!(" --pmroot \"{}\"", environment.pmroot_path));
    cmd.push_str(&format!(" --profile \"{}\"", profile.name));
    cmd.push_str(&format!(" --path \"{}\"", profile.prof_path));

    // Set all profiles devices as argument
    let devices = data.get_associated_devices_for_profile(profile);
    let found_devices = data
        .get_all_devices_of_profile(profile)
        .into_iter()
        .map(|index| devices.get(index).unwrap().clone())
        .collect::<Vec<Device>>();

    // Get only unique ones from found devices
    let devices = device::get_unique_devices(&found_devices);
    for dev in devices.iter() {
        if "PCI" != profile.prof_type {
            continue;
        }

        let bus_id = dev.sysfs_busid.replace(".", ":");
        let split = bus_id.split(":").collect::<Vec<_>>();
        let split_size = split.len();
        let bus_id = if split_size >= 3 {
            // Convert to int to remove leading 0
            format!(
                "{}:{}:{}",
                i64::from_str_radix(split[split_size - 3], 16).unwrap(),
                i64::from_str_radix(split[split_size - 2], 16).unwrap(),
                i64::from_str_radix(split[split_size - 1], 16).unwrap()
            )
        } else {
            dev.sysfs_busid.clone()
        };
        cmd.push_str(&format!(
            " --device \"{}|{}|{}|{}\"",
            dev.class_id, dev.vendor_id, dev.device_id, bus_id
        ));
    }
    cmd.push_str(" 2>&1");

    let status = Exec::shell(cmd).join();
    if status.is_err() || !status.unwrap().success() {
        return false;
    }

    // Only one database sync is required
    if ffi::Transaction::Install == transaction {
        data.environment.sync_package_manager_database = false;
    }
    false
}

pub fn prepare_autoconfigure(
    data: &Box<DataFFi>,
    args: &mut ffi::Arguments,
    operation: &str,
    autoconf_class_id: &str,
    autoconf_nonfree_driver: bool,
) -> Vec<String> {
    if !args.autoconfigure {
        return vec![];
    }

    let mut profiles_name = vec![];

    let devices = if "USB" == operation { &data.usb_devices } else { &data.pci_devices };
    let installed_profiles = if "USB" == operation {
        &data.installed_usb_profiles
    } else {
        &data.installed_pci_profiles
    };

    let mut found_device = false;
    for device in devices.iter() {
        if device.class_id != autoconf_class_id {
            continue;
        }
        found_device = true;
        let profile =
            device.available_profiles.iter().find(|x| autoconf_nonfree_driver || !x.is_nonfree);

        let device_info = format!(
            "{} ({}:{}:{}) {} {} {}",
            device.sysfs_busid,
            device.class_id,
            device.vendor_id,
            device.device_id,
            device.class_name,
            device.vendor_name,
            device.device_name
        );
        if profile.is_none() {
            print_warning(&format!("No config found for device: {device_info}"));
            continue;
        }
        let profile = profile.unwrap();

        // If force is not set, then we skip found profile
        let mut skip = false;
        if !args.force {
            skip = installed_profiles.iter().any(|x| x.name == profile.name);
        }

        // Print found profile
        if skip {
            print_status(&format!(
                "Skipping already installed profile '{}' for device: {}",
                profile.name, device_info
            ));
        } else {
            print_status(&format!("Using profile '{}' for device: {}", profile.name, device_info));
        }

        let profile_exists = profiles_name.iter().any(|x| x == &profile.name);
        if !profile_exists && !skip {
            profiles_name.push(profile.name.clone());
        }
    }

    if !found_device {
        print_warning(&format!("No device of class '{autoconf_class_id}' found!"));
    } else if !profiles_name.is_empty() {
        args.install = true;
    }

    profiles_name
}
