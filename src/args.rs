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

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Show PCI
    #[arg(long = "pci")]
    pub show_pci: bool,

    /// Install profile
    #[arg(short, long, value_name = "profile", conflicts_with("remove"))]
    pub install: Option<Vec<String>>,

    /// Remove profile
    #[arg(short, long, value_name = "profile", conflicts_with("install"))]
    pub remove: Option<Vec<String>>,

    /// Show detailed info for listings
    #[arg(short, long)]
    pub detail: bool,

    /// Force reinstall
    #[arg(short, long)]
    pub force: bool,

    /// List installed kernels
    #[arg(long)]
    pub list_installed: bool,

    /// List available profiles for all devices
    #[arg(long = "list")]
    pub list_available: bool,

    /// List all profiles
    #[arg(long)]
    pub list_all: bool,

    /// Autoconfigure
    #[arg(short, long, value_name = "classid", conflicts_with_all(["install", "remove"]))]
    pub autoconfigure: Option<Vec<String>>,

    /// Toggle AI SDK profiles
    #[arg(long = "ai_sdk")]
    pub is_ai_sdk: bool,

    #[arg(long, default_value_t = String::from("/var/cache/pacman/pkg"))]
    pub pmcachedir: String,
    #[arg(long, default_value_t = String::from("/etc/pacman.conf"))]
    pub pmconfig: String,
    #[arg(long, default_value_t = String::from("/"))]
    pub pmroot: String,
}
