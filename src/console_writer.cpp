/*
 *  This file is part of the mhwd - Manjaro Hardware Detection project
 *
 *  mhwd - Manjaro Hardware Detection
 *  Roland Singer <roland@manjaro.org>
 *  Łukasz Matysiak <december0123@gmail.com>
 *  Filipe Marques <eagle.software3@gmail.com>
 *
 *  Copyright (C) 2012 - 2016 Manjaro (http://manjaro.org)
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */
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

#include <hd.h>

#include <iostream>  // for cout
#include <numeric>   // for accumulate
#include <string>    // for string

#include <fmt/color.h>
#include <fmt/compile.h>
#include <fmt/core.h>

namespace mhwd::console_writer {

namespace {
inline void print_line() noexcept {
    fmt::print(FMT_COMPILE("{:->80}"), "\n");  // use '-' as a fill char
}

static constexpr auto CONSOLE_COLOR_RESET{"\033[m"};
static constexpr auto CONSOLE_TEXT_OUTPUT_COLOR{"\033[0;32m"};
}  // namespace

void print_status(const std::string_view& msg) {
    fmt::print(fg(fmt::color::red), fmt::format(FMT_COMPILE("> {}{}\n"), CONSOLE_COLOR_RESET, msg));
}

void print_error(const std::string_view& msg) {
    fmt::print(stderr, fg(fmt::color::red), fmt::format(FMT_COMPILE("Error: {}{}\n"), CONSOLE_COLOR_RESET, msg));
}

void print_warning(const std::string_view& msg) {
    fmt::print(fg(fmt::color::yellow), fmt::format(FMT_COMPILE("Warning: {}{}\n"), CONSOLE_COLOR_RESET, msg));
}

void print_message(mhwd::message_t type, const std::string_view& msg) {
    switch (type) {
    case mhwd::message_t::CONSOLE_OUTPUT:
        fmt::print("{}{}{}", CONSOLE_TEXT_OUTPUT_COLOR, msg, CONSOLE_COLOR_RESET);
        break;
    case mhwd::message_t::INSTALLDEPENDENCY_START:
        print_status("Installing dependency {} ...", msg);
        break;
    case mhwd::message_t::INSTALLDEPENDENCY_END:
        print_status("Successfully installed dependency  {}", msg);
        break;
    case mhwd::message_t::INSTALL_START:
        print_status("Installing {} ...", msg);
        break;
    case mhwd::message_t::INSTALL_END:
        print_status("Successfully installed {}", msg);
        break;
    case mhwd::message_t::REMOVE_START:
        print_status("Removing {} ...", msg);
        break;
    case mhwd::message_t::REMOVE_END:
        print_status("Successfully removed {}", msg);
        break;
    default:
        print_error("You shouldn't see this?! Unknown message type!");
        break;
    }
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
              << "  -ic/--installcustom <usb/pci> <path>\tinstall custom config(s)\n"
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
              << "Copyright (C) 2021 Manjaro Linux Developers\n"
              << "This is free software licensed under GNU GPL v3.0\n"
              << "FITNESS FOR A PARTICULAR PURPOSE.\n"
              << '\n';
}

void list_devices(const list_of_devices_t& devices, const std::string_view& type) {
    if (devices.empty()) {
        print_warning("No {} devices found!", type);
        return;
    }
    print_status("{} devices:", type);
    print_line();
    fmt::print(FMT_COMPILE("{:>30}{:>15}{:>8}{:>8}{:>8}{:>10}\n"), "TYPE", "BUS", "CLASS", "VENDOR", "DEVICE", "CONFIGS");
    print_line();
    for (const auto& device : devices) {
        fmt::print(FMT_COMPILE("{:>30}{:>15}{:>8}{:>8}{:>8}{:>10}\n"), device->class_name, device->sysfs_busid, device->class_id, device->vendor_id, device->device_id, device->available_configs.size());
    }
    fmt::print("\n\n");
}

void list_configs(const list_of_configs_t& configs, const std::string_view& header) {
    print_status(header);
    print_line();
    fmt::print(FMT_COMPILE("{:>24}{:>22}{:>18}{:>15}\n"), "NAME", "VERSION", "FREEDRIVER", "TYPE");
    print_line();
    for (const auto& config : configs) {
        fmt::print(FMT_COMPILE("{:>24}{:>22}{:>18}{:>15}\n"), config->name, config->version, config->is_freedriver, config->type);
    }
    fmt::print("\n\n");
}

void printAvailableConfigsInDetail(const std::string_view& device_type, const list_of_devices_t& devices) {
    bool config_found = false;
    for (const auto& device : devices) {
        if (device->available_configs.empty() && device->installed_configs.empty()) {
            continue;
        }
        config_found = true;

        print_line();
        print_status("{} Device: {} ({}:{}:{})", device_type, device->sysfs_id, device->class_id, device->vendor_id, device->device_id);
        fmt::print(FMT_COMPILE("  {} {} {}\n"), device->class_name, device->vendor_name, device->device_name);
        print_line();
        if (!device->installed_configs.empty()) {
            fmt::print("  > INSTALLED:\n\n");
            for (auto&& installed_config : device->installed_configs) {
                printConfigDetails(*installed_config);
            }
            fmt::print("\n\n");
        }
        if (!device->available_configs.empty()) {
            fmt::print("  > AVAILABLE:\n\n");
            for (auto&& available_config : device->available_configs) {
                printConfigDetails(*available_config);
            }
            fmt::print("\n");
        }
    }

    if (!config_found) {
        print_warning("no configs for {} devices found!", device_type);
    }
}

void printInstalledConfigs(const std::string_view& device_type, const list_of_configs_t& installed_configs) {
    if (installed_configs.empty()) {
        print_warning("no installed configs for {} devices found!", device_type);
        return;
    }
    for (const auto& config : installed_configs) {
        printConfigDetails(*config);
    }
    fmt::print("\n");
}

void printConfigDetails(const Config& config) noexcept {
    const auto& split_by_space = [](const auto& vec) {
        const auto& space_fold = [](auto&& lhs, const auto& rhs) {
            return rhs + ' ' + std::forward<decltype(lhs)>(lhs);
        };

        return vec.empty() ? "-" : std::accumulate(std::next(vec.begin()), vec.end(),
                   vec[0],  // start with first element
                   space_fold);
    };

    std::string class_ids{};
    std::string vendor_ids{};
    for (const auto& hwd : config.hwd_ids) {
        vendor_ids += split_by_space(hwd.vendor_ids);
        class_ids += split_by_space(hwd.class_ids);
    }
    const auto& dependencies = split_by_space(config.dependencies);
    const auto& conflicts    = split_by_space(config.conflicts);

    fmt::print(FMT_COMPILE("   NAME:\t{}\n   ATTACHED:\t{}\n   VERSION:\t{}\n   INFO:\t{}\n   PRIORITY:\t{}\n   FREEDRIVER:\t{}\n   DEPENDS:\t{}\n   CONFLICTS:\t{}\n   CLASSIDS:\t{}\n   VENDORIDS:\t{}\n\n"),
        config.name, config.type, config.version,
        (config.info.empty() ? "-" : config.info),
        config.priority, config.is_freedriver,
        dependencies,
        conflicts,
        class_ids, vendor_ids);
}

void printDeviceDetails(hw_item item, FILE* file_obj) noexcept {
    auto hd_data      = std::make_unique<hd_data_t>();
    auto* hd_list_obj = hd_list(hd_data.get(), item, 1, nullptr);

    for (hd_t* iter = hd_list_obj; iter != nullptr; iter = iter->next) {
        hd_dump_entry(hd_data.get(), iter, file_obj);
    }

    hd_free_hd_list(hd_list_obj);
    hd_free_hd_data(hd_data.get());
}

}  // namespace mhwd::console_writer
