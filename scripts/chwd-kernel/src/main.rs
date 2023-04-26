// Copyright (C) 2022-2023 Vladislav Nepogodin
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

mod kernel;

use clap::Parser;
use dialoguer::Confirm;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use subprocess::{Exec, Redirection};

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

fn new_alpm() -> alpm::Result<alpm::Alpm> {
    let pacman = pacmanconf::Config::with_opts(None, Some("/etc/pacman.conf"), Some("/")).unwrap();
    let alpm = alpm_utils::alpm_with_conf(&pacman)?;

    Ok(alpm)
}

fn simple_shell_exec(cmd: &str) -> String {
    let mut exec_out = Exec::shell(cmd).stdout(Redirection::Pipe).capture().unwrap().stdout_str();
    exec_out.pop();
    exec_out
}

#[inline]
fn get_kernel_running() -> String {
    simple_shell_exec(
        r#"(grep -Po '(?<=initrd\=\\initramfs-)(.+)(?=\.img)|(?<=boot\/vmlinuz-)([^ $]+)' /proc/cmdline)"#,
    )
}

fn root_check() {
    if nix::unistd::geteuid().is_root() {
        return;
    }
    occur_err("Please run as root.");
}

fn occur_err(msg: &str) {
    eprintln!("\x1B[31mError:\x1B[0m {}", msg);
    std::process::exit(1);
}

fn show_installed_kernels(kernels: &[kernel::Kernel]) {
    let current_kernel = get_kernel_running();
    println!(
        "\x1B[32mCurrently running:\x1B[0m {} ({})",
        simple_shell_exec("uname -r"),
        current_kernel
    );
    println!("The following kernels are installed in your system:");
    for kernel in kernels {
        if !kernel.is_installed().unwrap() {
            continue;
        }
        println!("local/{} {}", kernel.name, kernel.version().unwrap());
    }
}

fn show_available_kernels(kernels: &[kernel::Kernel]) {
    println!("\x1B[32mavailable kernels:\x1B[0m");
    for kernel in kernels {
        println!("{} {}", kernel.raw, kernel.version().unwrap());
    }
}

fn kernel_install(available_kernels: &[kernel::Kernel], kernel_names: &[String]) -> bool {
    let mut pkginstall = String::new();
    let mut rmc = false;

    let current_kernel = get_kernel_running();
    for kernel_name in kernel_names {
        if kernel_name == "rmc" {
            rmc = true;
            continue;
        } else if &current_kernel == kernel_name {
            occur_err(
                "You can't reinstall your current kernel. Please use 'pacman -Syu' instead to \
                 update.",
            );
        } else if available_kernels.iter().all(|elem| &elem.name != kernel_name) {
            eprintln!("\x1B[31mError:\x1B[0m Please make sure if the given kernel(s) exist(s).");
            show_available_kernels(available_kernels);
            return false;
        }

        pkginstall.push_str(&format!("{} ", &kernel_name));
    }
    let _ = Exec::shell("pacman -Syy").join();

    let outofdate = simple_shell_exec("pacman -Qqu | tr '\n' ' '");
    if !outofdate.is_empty() {
        eprintln!(
            "The following packages are out of date, please update your system first: {}",
            outofdate
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

    let exit_status = Exec::shell(format!("pacman -Syu {}", pkginstall)).join().unwrap();
    if rmc && exit_status.success() {
        let _ = Exec::shell(format!("pacman -R {}", current_kernel)).join();
    } else if rmc && !exit_status.success() {
        occur_err("\n'rmc' aborted because the kernel failed to install or canceled on removal.");
    }
    true
}

fn kernel_remove(available_kernels: &[kernel::Kernel], kernel_names: &[String]) -> bool {
    let mut pkgremove = String::new();

    let current_kernel = get_kernel_running();
    for kernel_name in kernel_names {
        if &current_kernel == kernel_name {
            occur_err("You can't remove your current kernel.");
        } else if !available_kernels
            .iter()
            .any(|elem| elem.is_installed().unwrap() && (&elem.name == kernel_name))
        {
            eprintln!("\x1B[31mError:\x1B[0m Kernel is not installed.");
            show_installed_kernels(available_kernels);
            return false;
        }

        pkgremove.push_str(&format!("{} ", kernel_name));
    }

    let exit_status = Exec::shell(format!("pacman -R {}", pkgremove)).join().unwrap();
    exit_status.success()
}

static ALPM_HANDLE: Lazy<Arc<Mutex<alpm::Alpm>>> = Lazy::new(|| {
    let alpm_handle = new_alpm().expect("Unable to initialize Alpm");

    Arc::new(Mutex::new(alpm_handle))
});

fn main() {
    let args = Args::parse();

    if args.list {
        let alpm_handle = &*ALPM_HANDLE.lock().unwrap();
        let kernels = kernel::get_kernels(alpm_handle);
        show_available_kernels(&kernels);
        return;
    }

    if args.list_installed {
        let alpm_handle = &*ALPM_HANDLE.lock().unwrap();
        let kernels = kernel::get_kernels(alpm_handle);
        show_installed_kernels(&kernels);
        return;
    }

    if args.running_kernel {
        println!("\x1B[32mrunning kernel:\x1B[0m '{}'", get_kernel_running());
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

            let alpm_handle = &*ALPM_HANDLE.lock().unwrap();
            let kernels = kernel::get_kernels(alpm_handle);
            kernel_install(&kernels, &args.install_kernels);
        },
        Some(WorkingMode::KernelRemove) => {
            root_check();

            let alpm_handle = &*ALPM_HANDLE.lock().unwrap();
            let kernels = kernel::get_kernels(alpm_handle);
            kernel_remove(&kernels, &args.remove_kernels);
        },
        _ => occur_err("Invalid argument (use -h for help)."),
    };
}
