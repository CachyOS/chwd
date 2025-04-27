// Copyright (C) 2023-2024 Vladislav Nepogodin
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

pub mod args;
pub mod console_writer;
pub mod device_misc;
pub mod localization;
pub mod logger;
pub mod misc;
pub mod profile_misc;

use chwd::profile::Profile;
use chwd::*;
use misc::Transaction;

use std::path::Path;
use std::sync::Arc;
use std::{fs, str};

use clap::Parser;
use i18n_embed::DesktopLanguageRequester;
use nix::unistd::Uid;
use subprocess::{Exec, Redirection};

fn main() -> anyhow::Result<()> {
    let requested_languages = DesktopLanguageRequester::requested_languages();
    let localizer = crate::localization::localizer();
    if let Err(error) = localizer.select(&requested_languages) {
        eprintln!("Error while loading languages for library_fluent {error}");
    }

    // initialize the logger
    logger::init_logger().expect("Failed to initialize logger");

    let args: Vec<String> = std::env::args().collect();

    // 1) Process arguments

    let mut argstruct = args::Args::parse();
    if args.len() <= 1 {
        argstruct.list_available = true;
    }

    let mut working_profiles: Vec<String> = vec![];

    let mut autoconf_class_id = String::new();

    if let Some(profile) = &argstruct.install {
        working_profiles.push(profile.to_lowercase());
    }

    if let Some(profile) = &argstruct.remove {
        working_profiles.push(profile.to_lowercase());
    }

    if let Some(class_id) = &argstruct.autoconfigure {
        autoconf_class_id = class_id.to_lowercase();
    }

    // 2) Initialize
    let mut data_obj = data::Data::new(argstruct.is_ai_sdk);

    let missing_dirs = misc::check_environment();
    if !missing_dirs.is_empty() {
        log::error!("Following directories do not exist:");
        for missing_dir in missing_dirs.iter() {
            log::info!("{missing_dir}");
        }
        anyhow::bail!("Error occurred");
    }

    // 3) Perform operations
    console_writer::handle_arguments_listing(&data_obj, &argstruct);

    // 4) Auto configuration
    let mut prepared_profiles =
        prepare_autoconfigure(&data_obj, &mut argstruct, &autoconf_class_id);
    working_profiles.append(&mut prepared_profiles);

    // Transaction
    if !(argstruct.install.is_some() || argstruct.remove.is_some()) {
        return Ok(());
    }
    if !Uid::effective().is_root() {
        console_writer::print_error_msg!("root-operation");
        anyhow::bail!("Error occurred");
    }

    for profile_name in working_profiles.iter() {
        if argstruct.install.is_some() {
            let working_profile = get_available_profile(&mut data_obj, profile_name);
            if working_profile.is_none() {
                let working_profile = get_db_profile(&data_obj, profile_name);
                if working_profile.is_none() {
                    console_writer::print_error_msg!(
                        "profile-not-exist",
                        profile_name = profile_name
                    );
                    anyhow::bail!("Error occurred");
                }
                console_writer::print_error_msg!("no-matching-device", profile_name = profile_name);
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
                console_writer::print_error_msg!(
                    "profile-not-installed",
                    profile_name = profile_name
                );
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
    args: &mut args::Args,
    autoconf_class_id: &str,
) -> Vec<String> {
    if args.autoconfigure.is_none() {
        return vec![];
    }

    let mut profiles_name = vec![];

    let devices = &data.pci_devices;
    let installed_profiles = &data.installed_profiles;

    let mut found_device = false;
    for device in devices.iter() {
        if autoconf_class_id != "any" && device.class_id != autoconf_class_id {
            continue;
        }
        found_device = true;
        let profile = device.available_profiles.first();

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
            if autoconf_class_id != "any" {
                log::warn!("No config found for device: {device_info}");
            }
            continue;
        }
        let profile = profile.unwrap();

        // If force is not set, then we skip found profile
        let skip = !args.force && installed_profiles.iter().any(|x| x.name == profile.name);

        // Print found profile
        if skip {
            log::info!(
                "Skipping already installed profile '{}' for device: {device_info}",
                profile.name
            );
        } else {
            log::info!("Using profile '{}' for device: {device_info}", profile.name);
        }

        let profile_exists = profiles_name.iter().any(|x| x == &profile.name);
        if !profile_exists && !skip {
            profiles_name.push(profile.name.clone());
        }
    }

    if !found_device {
        log::warn!("No device of class '{autoconf_class_id}' found!");
    } else if !profiles_name.is_empty() {
        args.install = Some(profiles_name.first().unwrap().clone());
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
    let all_profiles = &data.all_profiles;
    misc::find_profile(profile_name, all_profiles)
}

fn get_installed_profile(data: &data::Data, profile_name: &str) -> Option<Arc<Profile>> {
    // Get the right profiles
    let installed_profiles = &data.installed_profiles;
    misc::find_profile(profile_name, installed_profiles)
}

pub fn run_script(
    data: &mut data::Data,
    args: &args::Args,
    profile: &Profile,
    transaction: Transaction,
) -> bool {
    let mut cmd_args: Vec<String> = if Transaction::Remove == transaction {
        vec!["--remove".into()]
    } else {
        vec!["--install".into()]
    };

    if data.sync_package_manager_database {
        cmd_args.push("--sync".into());
    }

    cmd_args.extend_from_slice(&["--cachedir".into(), args.pmcachedir.clone()]);
    cmd_args.extend_from_slice(&["--pmconfig".into(), args.pmconfig.clone()]);
    cmd_args.extend_from_slice(&["--pmroot".into(), args.pmroot.clone()]);
    cmd_args.extend_from_slice(&["--profile".into(), profile.name.clone()]);
    cmd_args.extend_from_slice(&["--path".into(), profile.prof_path.clone()]);

    let status =
        Exec::cmd(consts::CHWD_SCRIPT_PATH).args(&cmd_args).stderr(Redirection::Merge).join();
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
    args: &args::Args,
    profile: &Arc<Profile>,
    transaction_type: Transaction,
    force: bool,
) -> bool {
    let status = perform_transaction_type(data, args, profile, transaction_type, force);

    let profile_name = &profile.name;
    match status {
        misc::Status::ErrorNotInstalled => {
            console_writer::print_error_msg!("profile-not-installed", profile_name = profile_name)
        },
        misc::Status::ErrorAlreadyInstalled => log::warn!(
            "a version of profile '{profile_name}' is already installed!\nUse -f/--force to force \
             installation...",
        ),
        misc::Status::ErrorNoMatchLocalConfig => {
            console_writer::print_error_msg!("pass-profile-no-match-install")
        },
        misc::Status::ErrorScriptFailed => console_writer::print_error_msg!("script-failed"),
        misc::Status::ErrorSetDatabase => console_writer::print_error_msg!("failed-set-db"),
        _ => (),
    }

    data.update_installed_profile_data();

    misc::Status::Success == status
}

fn perform_transaction_type(
    data_obj: &mut data::Data,
    args: &args::Args,
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

fn install_profile(data: &mut data::Data, args: &args::Args, profile: &Profile) -> misc::Status {
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

fn remove_profile(data: &mut data::Data, args: &args::Args, profile: &Profile) -> misc::Status {
    let installed_profile = get_installed_profile(data, &profile.name);

    // Check if installed
    if installed_profile.is_none() {
        return misc::Status::ErrorNotInstalled;
    }
    // Run script
    if !run_script(data, args, installed_profile.as_ref().unwrap(), Transaction::Remove) {
        return misc::Status::ErrorScriptFailed;
    }

    if !profile::remove_profile_from_file(&profile.prof_path, &profile.name) {
        return misc::Status::ErrorSetDatabase;
    }

    data.update_installed_profile_data();
    misc::Status::Success
}
