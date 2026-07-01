// Copyright (C) 2022-2026 Vladislav Nepogodin
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

use chwd_kernel::kernel::{self, KernelPkg};

use clap::Parser;
use dialoguer::Confirm;
use itertools::Itertools;
use subprocess::Exec;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(arg_required_else_help(true))]
struct Args {
    /// List available kernels
    #[arg(long)]
    list: bool,

    /// List installed kernels
    #[arg(long)]
    list_installed: bool,

    /// Show running kernel
    #[arg(long)]
    running_kernel: bool,

    /// Install kernels
    #[arg(short, long = "install", conflicts_with("remove_kernels"))]
    install_kernels: Vec<String>,

    /// Uninstall kernels
    #[arg(short, long = "remove", conflicts_with("install_kernels"))]
    remove_kernels: Vec<String>,
}

enum WorkingMode {
    KernelInstall,
    KernelRemove,
}

fn root_check() {
    if nix::unistd::geteuid().is_root() {
        return;
    }
    occur_err("Please run as root.");
}

fn occur_err(msg: &str) {
    eprintln!("\x1B[31mError:\x1B[0m {msg}");
    std::process::exit(1);
}

fn show_installed_kernels(kernels: &[KernelPkg]) {
    println!(
        "\x1B[32mCurrently running:\x1B[0m {} ({})",
        kernel::shell_capture("uname -r"),
        kernel::running_kernel()
    );
    println!("The following kernels are installed in your system:");
    for kernel in kernels.iter().filter(|k| k.installed).unique_by(|k| &k.name) {
        println!("local/{} {}", kernel.name, kernel.version);
    }
}

fn show_available_kernels(kernels: &[KernelPkg]) {
    println!("\x1B[32mavailable kernels:\x1B[0m");
    for kernel in kernels {
        println!("{} {}", kernel.raw, kernel.version);
    }
}

/// Validate the requested kernel names and map them to `repo/name` identities.
fn resolve_selection(
    available_kernels: &[KernelPkg],
    kernel_names: &[String],
    removing: bool,
    current_kernel: &str,
) -> Option<Vec<String>> {
    let mut raws = Vec::new();
    for kernel_name in kernel_names {
        if !removing && kernel_name == "rmc" {
            continue;
        }
        if current_kernel == kernel_name {
            let current_installed =
                available_kernels.iter().any(|k| k.name == *kernel_name && k.installed);
            if removing {
                occur_err("You can't remove your current kernel.");
            } else if current_installed {
                // Reinstalling the running kernel is a noop
                occur_err(
                    "You can't reinstall your current kernel. Please use 'pacman -Syu' instead to \
                     update.",
                );
            }
        }

        let matched = if removing {
            available_kernels.iter().find(|k| k.installed && k.name == *kernel_name)
        } else {
            available_kernels.iter().find(|k| k.name == *kernel_name)
        };

        match matched {
            Some(kernel) => raws.push(kernel.raw.clone()),
            None if removing => {
                eprintln!("\x1B[31mError:\x1B[0m Kernel is not installed.");
                show_installed_kernels(available_kernels);
                return None;
            },
            None => {
                eprintln!(
                    "\x1B[31mError:\x1B[0m Please make sure if the given kernel(s) exist(s)."
                );
                show_available_kernels(available_kernels);
                return None;
            },
        }
    }
    Some(raws)
}

fn kernel_install(
    handle: &alpm::Alpm,
    available_kernels: &[KernelPkg],
    kernel_names: &[String],
) -> bool {
    let current_kernel = kernel::running_kernel();
    let rmc = kernel_names.iter().any(|name| name == "rmc");

    let Some(install_raws) =
        resolve_selection(available_kernels, kernel_names, false, &current_kernel)
    else {
        return false;
    };

    let plan = kernel::resolve_transaction(handle, available_kernels, &install_raws, &[]);
    let pkginstall = plan.pacman_install.join(" ");

    let outofdate = kernel::shell_capture("checkupdates");
    if !outofdate.is_empty() {
        eprintln!(
            "The following packages are out of date, please update your system first:\n{outofdate}"
        );
        if !Confirm::new()
            .with_prompt("Do you want to continue anyway?")
            .default(true)
            .interact()
            .unwrap()
        {
            return false;
        }
    }

    let exit_status = Exec::shell(format!("pacman -Syu {pkginstall}")).join().unwrap();
    if rmc && exit_status.success() {
        let _ = Exec::shell(format!("pacman -Rsn {current_kernel}")).join();
    } else if rmc && !exit_status.success() {
        occur_err("\n'rmc' aborted because the kernel failed to install or canceled on removal.");
    }
    true
}

fn kernel_remove(
    handle: &alpm::Alpm,
    available_kernels: &[KernelPkg],
    kernel_names: &[String],
) -> bool {
    let current_kernel = kernel::running_kernel();

    let Some(remove_raws) =
        resolve_selection(available_kernels, kernel_names, true, &current_kernel)
    else {
        return false;
    };

    let plan = kernel::resolve_transaction(handle, available_kernels, &[], &remove_raws);
    let pkgremove = plan.pacman_remove.join(" ");

    let exit_status = Exec::shell(format!("pacman -Rsn {pkgremove}")).join().unwrap();
    exit_status.success()
}

fn init_alpm_handle() -> alpm::Alpm {
    kernel::open_alpm().expect("Unable to initialize Alpm")
}

fn main() {
    let args = Args::parse();

    if args.list {
        let alpm_handle = init_alpm_handle();
        let kernels = kernel::get_kernels(&alpm_handle);
        show_available_kernels(&kernels);
        return;
    }

    if args.list_installed {
        let alpm_handle = init_alpm_handle();
        let kernels = kernel::get_kernels(&alpm_handle);
        show_installed_kernels(&kernels);
        return;
    }

    if args.running_kernel {
        println!("\x1B[32mrunning kernel:\x1B[0m '{}'", kernel::running_kernel());
        return;
    }

    let working_mode = if !args.install_kernels.is_empty() {
        Some(WorkingMode::KernelInstall)
    } else if !args.remove_kernels.is_empty() {
        Some(WorkingMode::KernelRemove)
    } else {
        None
    };

    match working_mode {
        Some(WorkingMode::KernelInstall) => {
            root_check();

            let alpm_handle = init_alpm_handle();
            let kernels = kernel::get_kernels(&alpm_handle);
            kernel_install(&alpm_handle, &kernels, &args.install_kernels);
        },
        Some(WorkingMode::KernelRemove) => {
            root_check();

            let alpm_handle = init_alpm_handle();
            let kernels = kernel::get_kernels(&alpm_handle);
            kernel_remove(&alpm_handle, &kernels, &args.remove_kernels);
        },
        _ => occur_err("Invalid argument (use -h for help)."),
    }
}
