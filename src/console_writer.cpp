//
// Copyright (C) 2022-2023 Vladislav Nepogodin
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

#include "console_writer.hpp"

#if defined(__clang__)
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdollar-in-identifier-extension"
#pragma clang diagnostic ignored "-Wsign-conversion"
#elif defined(__GNUC__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wsign-conversion"
#endif

#include "chwd-cxxbridge/lib.h"

#if defined(__clang__)
#pragma clang diagnostic pop
#elif defined(__GNUC__)
#pragma GCC diagnostic pop
#endif

#include <iostream>  // for cout
#include <string>    // for string

#include <fmt/color.h>
#include <fmt/compile.h>
#include <fmt/core.h>

namespace mhwd::console_writer {

void print_status(const std::string_view& msg) {
    chwd::print_status(msg.data());
}

void print_error(const std::string_view& msg) {
    chwd::print_error(msg.data());
}

void print_warning(const std::string_view& msg) {
    chwd::print_warning(msg.data());
}

void print_message(chwd::Message type, const std::string_view& msg) {
    chwd::print_message(type, msg.data());
}

void print_help() noexcept {
    std::cout << "Usage: mhwd [OPTIONS] <config(s)>\n\n"
              << "  --pci\t\t\t\t\tlist only pci devices and driver configs\n"
              << "  --usb\t\t\t\t\tlist only usb devices and driver configs\n"
              << "  -h/--help\t\t\t\tshow help\n"
              << "  -v/--version\t\t\t\tshow version of mhwd\n"
              << "  --is_nvidia_card\t\t\tcheck if the nvidia card found\n"
              << "  -f/--force\t\t\t\tforce reinstallation\n"
              << "  -d/--detail\t\t\t\tshow detailed info for -l/-li/-lh\n"
              << "  -l/--list\t\t\t\tlist available configs for devices\n"
              << "  -la/--listall\t\t\t\tlist all driver configs\n"
              << "  -li/--listinstalled\t\t\tlist installed driver configs\n"
              << "  -lh/--listhardware\t\t\tlist hardware information\n"
              << "  -i/--install <usb/pci> <config(s)>\tinstall driver config(s)\n"
              << "  -r/--remove <usb/pci> <config(s)>\tremove driver config(s)\n"
              << "  -a/--auto <usb/pci> <free/nonfree> <classid>\tauto install configs for classid\n"
              << "  --pmcachedir <path>\t\t\tset package manager cache path\n"
              << "  --pmconfig <path>\t\t\tset package manager config\n"
              << "  --pmroot <path>\t\t\tset package manager root\n"
              << '\n';
}

void print_version(const std::string_view& version_mhwd, const std::string_view& year_copy) noexcept {
    std::cout << "CachyOS Hardware Detection v" << version_mhwd << "\n\n"
              << "Copyright (C) " << year_copy << " CachyOS Developers\n"
              << "This is free software licensed under GNU GPL v3.0\n"
              << "FITNESS FOR A PARTICULAR PURPOSE.\n"
              << '\n';
}

}  // namespace mhwd::console_writer
