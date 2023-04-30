// Copyright (C) 2023 Vladislav Nepogodin
//
// This file is part of CachyOS chwd.
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

pub mod console_writer;
pub mod consts;
pub mod data;
pub mod device;
pub mod misc;
pub mod profile;

use misc::Transaction;
use profile::Profile;
use std::fs;

use clap::Parser;
use nix::unistd::Uid;
use std::path::Path;
use std::sync::Arc;
use subprocess::Exec;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Show PCI
    #[arg(long = "pci")]
    show_pci: bool,

    /// Install profile
    #[arg(short, long, number_of_values = 2, value_names = &["usb/pci", "profile"], conflicts_with("remove"))]
    install: Option<Vec<String>>,

    /// Remove profile
    #[arg(short, long, number_of_values = 2, value_names = &["usb/pci", "profile"], conflicts_with("install"))]
    remove: Option<Vec<String>>,

    /// Show detailed info for listings
    #[arg(short, long)]
    detail: bool,

    /// Force reinstall
    #[arg(short, long)]
    force: bool,

    /// List installed kernels
    #[arg(long)]
    list_installed: bool,

    /// List available profiles for all devices
    #[arg(long = "list")]
    list_available: bool,

    /// List all profiles
    #[arg(long)]
    list_all: bool,

    /// Autoconfigure
    #[arg(short, long, number_of_values = 3, value_names = &["usb/pci", "free/nonfree", "classid"], conflicts_with_all(["install", "remove"]))]
    autoconfigure: Option<Vec<String>>,

    /// Print if nvidia card found
    #[arg(long = "is_nvidia_card")]
    is_nvidia_card: bool,

    #[arg(long, default_value_t = String::from("/var/cache/pacman/pkg"))]
    pmcachedir: String,
    #[arg(long, default_value_t = String::from("/etc/pacman.conf"))]
    pmconfig: String,
    #[arg(long, default_value_t = String::from("/"))]
    pmroot: String,
}

fn perceed_inst_rem(
    args: &Option<Vec<String>>,
    operation: &mut String,
    working_profiles: &mut Vec<String>,
) -> anyhow::Result<()> {
    if let Some(values) = args {
        let device_type = values[0].clone();
        let profile = values[1].to_lowercase();

        if "pci" != device_type && "usb" != device_type {
            anyhow::bail!("invalid use of option: {args:?}");
        }
        *operation = device_type.to_uppercase();
        working_profiles.push(profile);
    }

    Ok(())
}

fn perceed_autoconf(
    args: &Option<Vec<String>>,
    operation: &mut String,
    autoconf_class_id: &mut String,
    is_nonfree: &mut bool,
) -> anyhow::Result<()> {
    if let Some(values) = args {
        let device_type = values[0].clone();
        let driver_type = values[1].clone();
        *is_nonfree = "nonfree" == driver_type;
        *autoconf_class_id = values[2].to_lowercase();

        if ("pci" != device_type && "usb" != device_type)
            || ("free" != driver_type && "nonfree" != driver_type)
        {
            anyhow::bail!("invalid use of option: {args:?}");
        }
        *operation = device_type.to_uppercase();
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    // 1) Process arguments

    let mut argstruct = Args::parse();
    if std::env::args().len() <= 1 {
        argstruct.list_available = true;
    }

    if argstruct.is_nvidia_card {
        misc::check_nvidia_card();
        return Ok(());
    }

    let mut working_profiles: Vec<String> = vec![];

    let mut operation = String::new();
    let mut autoconf_class_id = String::new();
    let mut autoconf_nonfree_driver = false;
    perceed_autoconf(
        &argstruct.autoconfigure,
        &mut operation,
        &mut autoconf_class_id,
        &mut autoconf_nonfree_driver,
    )?;
    perceed_inst_rem(&argstruct.install, &mut operation, &mut working_profiles)?;
    perceed_inst_rem(&argstruct.remove, &mut operation, &mut working_profiles)?;

    if !argstruct.show_pci {
        argstruct.show_pci = true;
    }

    // 2) Initialize
    let mut data_obj = data::Data::new();

    let missing_dirs = misc::check_environment();
    if !missing_dirs.is_empty() {
        console_writer::print_error("Following directories do not exist:");
        for missing_dir in missing_dirs.iter() {
            console_writer::print_status(missing_dir);
        }
        anyhow::bail!("Error occurred");
    }

    // 3) Perform operations
    console_writer::handle_arguments_listing(&data_obj, &argstruct);

    // 4) Auto configuration
    let mut prepared_profiles = prepare_autoconfigure(
        &data_obj,
        &mut argstruct,
        &autoconf_class_id,
        autoconf_nonfree_driver,
    );
    working_profiles.append(&mut prepared_profiles);

    // Transaction
    if !(argstruct.install.is_some() || argstruct.remove.is_some()) {
        return Ok(());
    }
    if !Uid::effective().is_root() {
        console_writer::print_error("You cannot perform this operation unless you are root!");
        anyhow::bail!("Error occurred");
    }

    for profile_name in working_profiles.iter() {
        if argstruct.install.is_some() {
            let working_profile = get_available_profile(&mut data_obj, profile_name);
            if working_profile.is_none() {
                let working_profile = get_db_profile(&data_obj, profile_name);
                if working_profile.is_none() {
                    console_writer::print_error(&format!(
                        "profile '{profile_name}' does not exist!"
                    ));
                    anyhow::bail!("Error occurred");
                }
                console_writer::print_error(&format!(
                    "no matching device for profile '{profile_name}' found!"
                ));
                anyhow::bail!("Error occurred");
            }

            if !perform_transaction(
                &mut data_obj,
                &argstruct,
                &working_profile.unwrap(),
                Transaction::Install,
                argstruct.force,
            ) {
                anyhow::bail!("Error occurred");
            }
        } else if argstruct.remove.is_some() {
            let working_profile = get_installed_profile(&data_obj, profile_name);
            if working_profile.is_none() {
                console_writer::print_error(&format!("profile '{profile_name}' is not installed!"));
                anyhow::bail!("Error occurred");
            } else if !perform_transaction(
                &mut data_obj,
                &argstruct,
                working_profile.as_ref().unwrap(),
                Transaction::Remove,
                argstruct.force,
            ) {
                anyhow::bail!("Error occurred");
            }
        }
    }

    Ok(())
}

fn prepare_autoconfigure(
    data: &data::Data,
    args: &mut Args,
    autoconf_class_id: &str,
    autoconf_nonfree_driver: bool,
) -> Vec<String> {
    if args.autoconfigure.is_none() {
        return vec![];
    }

    let mut profiles_name = vec![];

    let devices = &data.pci_devices;
    let installed_profiles = &data.installed_pci_profiles;

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
            console_writer::print_warning(&format!("No config found for device: {device_info}"));
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
            console_writer::print_status(&format!(
                "Skipping already installed profile '{}' for device: {}",
                profile.name, device_info
            ));
        } else {
            console_writer::print_status(&format!(
                "Using profile '{}' for device: {}",
                profile.name, device_info
            ));
        }

        let profile_exists = profiles_name.iter().any(|x| x == &profile.name);
        if !profile_exists && !skip {
            profiles_name.push(profile.name.clone());
        }
    }

    if !found_device {
        console_writer::print_warning(&format!("No device of class '{autoconf_class_id}' found!"));
    } else if !profiles_name.is_empty() {
        args.install = Some(profiles_name.clone());
    }

    profiles_name
}

fn get_available_profile(data: &mut data::Data, profile_name: &str) -> Option<Arc<Profile>> {
    // Get the right devices
    let devices = &mut data.pci_devices;

    for device in devices.iter_mut() {
        let available_profiles = &mut device.available_profiles;
        if available_profiles.is_empty() {
            continue;
        }

        let available_profile = available_profiles.iter_mut().find(|x| x.name == profile_name);
        if let Some(available_profile) = available_profile {
            return Some(Arc::clone(available_profile));
        }
    }
    None
}

fn get_db_profile(data: &data::Data, profile_name: &str) -> Option<Arc<Profile>> {
    // Get the right profiles
    let all_profiles = &data.all_pci_profiles;
    misc::find_profile(profile_name, all_profiles)
}

fn get_installed_profile(data: &data::Data, profile_name: &str) -> Option<Arc<Profile>> {
    // Get the right profiles
    let installed_profiles = &data.installed_pci_profiles;
    misc::find_profile(profile_name, installed_profiles)
}

pub fn run_script(
    data: &mut data::Data,
    args: &Args,
    profile: &Profile,
    transaction: Transaction,
) -> bool {
    let mut cmd = format!("exec {}", consts::CHWD_SCRIPT_PATH);

    if Transaction::Remove == transaction {
        cmd.push_str(" --remove");
    } else {
        cmd.push_str(" --install");
    }

    if data.sync_package_manager_database {
        cmd.push_str(" --sync");
    }

    cmd.push_str(&format!(" --cachedir \"{}\"", args.pmcachedir));
    cmd.push_str(&format!(" --pmconfig \"{}\"", args.pmconfig));
    cmd.push_str(&format!(" --pmroot \"{}\"", args.pmroot));
    cmd.push_str(&format!(" --profile \"{}\"", profile.name));
    cmd.push_str(&format!(" --path \"{}\"", profile.prof_path));

    // Set all profiles devices as argument
    let devices = &data.pci_devices;
    let found_devices = data::get_all_devices_of_profile(&data.pci_devices, profile)
        .into_iter()
        .map(|index| devices.get(index).unwrap().clone())
        .collect::<Vec<device::Device>>();

    // Get only unique ones from found devices
    let devices = device::get_unique_devices(&found_devices);
    for dev in devices.iter() {
        let bus_id = dev.sysfs_busid.replace('.', ":");
        let split = bus_id.split(':').collect::<Vec<_>>();
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
    if Transaction::Install == transaction {
        data.sync_package_manager_database = false;
    }
    true
}

fn perform_transaction(
    data: &mut data::Data,
    args: &Args,
    profile: &Arc<Profile>,
    transaction_type: Transaction,
    force: bool,
) -> bool {
    let status = perform_transaction_type(data, args, profile, transaction_type, force);

    match status {
        misc::Status::ErrorNotInstalled => {
            console_writer::print_error(&format!("profile '{}' is not installed!", &profile.name))
        },
        misc::Status::ErrorAlreadyInstalled => console_writer::print_warning(&format!(
            "a version of profile '{}' is already installed!\nUse -f/--force to force \
             installation...",
            &profile.name
        )),
        misc::Status::ErrorNoMatchLocalConfig => {
            console_writer::print_error("passed profile does not match with installed profile!")
        },
        misc::Status::ErrorScriptFailed => console_writer::print_error("script failed!"),
        misc::Status::ErrorSetDatabase => console_writer::print_error("failed to set database!"),
        _ => (),
    }

    data.update_installed_profile_data();

    misc::Status::Success == status
}

fn perform_transaction_type(
    data_obj: &mut data::Data,
    args: &Args,
    profile: &Arc<Profile>,
    transaction_type: Transaction,
    force: bool,
) -> misc::Status {
    // Check if already installed
    let installed_profile = get_installed_profile(data_obj, &profile.name);
    let mut status = misc::Status::Success;

    if (Transaction::Remove == transaction_type) || (installed_profile.is_some() && force) {
        if installed_profile.is_none() {
            return misc::Status::ErrorNotInstalled;
        }
        console_writer::print_message(
            misc::Message::RemoveStart,
            &installed_profile.as_ref().unwrap().name,
        );
        status = remove_profile(data_obj, args, installed_profile.as_ref().unwrap());
        if misc::Status::Success != status {
            return status;
        }
        console_writer::print_message(
            misc::Message::RemoveEnd,
            &installed_profile.as_ref().unwrap().name,
        );
    }

    if Transaction::Install == transaction_type {
        // Check if already installed but not allowed to reinstall
        if installed_profile.is_some() && !force {
            return misc::Status::ErrorAlreadyInstalled;
        }
        console_writer::print_message(misc::Message::InstallStart, &profile.name);
        status = install_profile(data_obj, args, profile);
        if misc::Status::Success != status {
            return status;
        }
        console_writer::print_message(misc::Message::InstallEnd, &profile.name);
    }
    status
}

fn install_profile(data: &mut data::Data, args: &Args, profile: &Profile) -> misc::Status {
    if !run_script(data, args, profile, Transaction::Install) {
        return misc::Status::ErrorScriptFailed;
    }

    let db_dir = consts::CHWD_PCI_DATABASE_DIR;
    let working_dir = format!(
        "{}/{}",
        db_dir,
        Path::new(&profile.prof_path).parent().unwrap().file_name().unwrap().to_str().unwrap()
    );
    let _ = fs::create_dir_all(&working_dir);
    if !profile::write_profile_to_file(
        &format!("{}/{}", &working_dir, consts::CHWD_CONFIG_FILE),
        profile,
    ) {
        return misc::Status::ErrorSetDatabase;
    }

    // Note: installed profile vectors have to be updated manually with
    // update_installed_profile_data(Data)
    misc::Status::Success
}

fn remove_profile(data: &mut data::Data, args: &Args, profile: &Profile) -> misc::Status {
    let installed_profile = get_installed_profile(data, &profile.name);

    // Check if installed
    if installed_profile.is_none() {
        return misc::Status::ErrorNotInstalled;
    }
    // Run script
    if !run_script(data, args, installed_profile.as_ref().unwrap(), Transaction::Remove) {
        return misc::Status::ErrorScriptFailed;
    }

    if fs::remove_file(&profile.prof_path).is_err() {
        return misc::Status::ErrorSetDatabase;
    }

    data.update_installed_profile_data();
    misc::Status::Success
}
