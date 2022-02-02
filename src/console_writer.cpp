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
// Copyright (C) 2022 Vladislav Nepogodin
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

#include <iomanip>
#include <iostream>
#include <memory>
#include <string>
#include <vector>

#include <fmt/color.h>
#include <fmt/core.h>

namespace mhwd {

void ConsoleWriter::print_status(const std::string_view& msg) const {
    fmt::print(fg(fmt::color::red), "> {}\n", msg);
}

void ConsoleWriter::print_error(const std::string_view& msg) const {
    fmt::print(stderr, fg(fmt::color::red), "Error: {}\n", msg);
}

void ConsoleWriter::print_warning(const std::string_view& msg) const {
    fmt::print(fg(fmt::color::red), "Warning: {}\n", msg);
}

void ConsoleWriter::print_message(mhwd::message_t type, const std::string_view& msg) const {
    switch (type) {
    case mhwd::message_t::CONSOLE_OUTPUT:
        print_status(fmt::format("{}{}{}", CONSOLE_TEXT_OUTPUT_COLOR, msg, CONSOLE_COLOR_RESET));
        break;
    case mhwd::message_t::INSTALLDEPENDENCY_START:
        print_status(fmt::format("Installing dependency {} ...", msg));
        break;
    case mhwd::message_t::INSTALLDEPENDENCY_END:
        print_status(fmt::format("Successfully installed dependency  {}", msg));
        break;
    case mhwd::message_t::INSTALL_START:
        print_status(fmt::format("Installing {} ...", msg));
        break;
    case mhwd::message_t::INSTALL_END:
        print_status(fmt::format("Successfully installed {}", msg));
        break;
    case mhwd::message_t::REMOVE_START:
        print_status(fmt::format("Removing {} ...", msg));
        break;
    case mhwd::message_t::REMOVE_END:
        print_status(fmt::format("Successfully removed {}", msg));
        break;
    default:
        print_error("You shouldn't see this?! Unknown message type!");
        break;
    }
}

void ConsoleWriter::print_help() const {
    std::cout << "Usage: mhwd [OPTIONS] <config(s)>\n\n"
              << "  --pci\t\t\t\t\tlist only pci devices and driver configs\n"
              << "  --usb\t\t\t\t\tlist only usb devices and driver configs\n"
              << "  -h/--help\t\t\t\tshow help\n"
              << "  -v/--version\t\t\t\tshow version of mhwd\n"
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

void ConsoleWriter::print_version(const std::string_view& version_mhwd, const std::string_view& year_copy) const {
    std::cout << "Manjaro Hardware Detection v" << version_mhwd << "\n\n"
              << "Copyright (C) " << year_copy << " Manjaro Linux Developers\n"
              << "This is free software licensed under GNU GPL v3.0\n"
              << "FITNESS FOR A PARTICULAR PURPOSE.\n"
              << '\n';
}

void ConsoleWriter::list_devices(const std::vector<std::shared_ptr<Device>>& devices, std::string type) const {
    if (devices.empty()) {
        print_warning(fmt::format("No {} devices found!", type));
        return;
    }
    print_status(fmt::format("{} devices:", type));
    printLine();
    std::cout << std::setw(30) << "TYPE"
              << std::setw(15) << "BUS"
              << std::setw(8) << "CLASS"
              << std::setw(8) << "VENDOR"
              << std::setw(8) << "DEVICE"
              << std::setw(10) << "CONFIGS" << '\n';
    printLine();
    for (const auto& device : devices) {
        std::cout << std::setw(30) << device->class_name
                  << std::setw(15) << device->sysfs_busid
                  << std::setw(8) << device->class_id
                  << std::setw(8) << device->vendor_id
                  << std::setw(8) << device->device_id
                  << std::setw(10) << device->available_configs.size() << '\n';
    }
    fmt::print("\n\n");
}

void ConsoleWriter::list_configs(const std::vector<std::shared_ptr<Config>>& configs, std::string header) const {
    print_status(header);
    printLine();
    std::cout << std::setw(22) << "NAME"
              << std::setw(22) << "VERSION"
              << std::setw(20) << "FREEDRIVER"
              << std::setw(15) << "TYPE" << '\n';
    printLine();
    for (const auto& config : configs) {
        std::cout << std::setw(22) << config->name
                  << std::setw(22) << config->version
                  << std::setw(20) << std::boolalpha << config->is_freedriver
                  << std::setw(15) << config->type << '\n';
    }
    fmt::print("\n\n");
}

void ConsoleWriter::printAvailableConfigsInDetail(const std::string_view& device_type, const std::vector<std::shared_ptr<Device>>& devices) const {
    bool config_found = false;
    for (const auto& device : devices) {
        if (device->available_configs.empty() && device->installed_configs.empty()) {
            continue;
        }
        config_found = true;

        printLine();
        print_status(fmt::format("{} Device: {} ({}:{}:{})", device_type, device->sysfs_id, device->class_id, device->vendor_id, device->device_id));
        fmt::print("  {} {} {}\n", device->class_name, device->vendor_name, device->device_name);
        printLine();
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
        print_warning(fmt::format("no configs for {} devices found!", device_type));
    }
}

void ConsoleWriter::printInstalledConfigs(const std::string_view& device_type,
    const std::vector<std::shared_ptr<Config>>& installed_configs) const {
    if (installed_configs.empty()) {
        print_warning(fmt::format("no installed configs for {} devices found!", device_type));
        return;
    }
    for (const auto& config : installed_configs) {
        printConfigDetails(*config);
    }
    fmt::print("\n");
}

void ConsoleWriter::printConfigDetails(const Config& config) const {
    std::string classids;
    std::string vendorids;
    for (const auto& hwd : config.hwd_ids) {
        for (const auto& m_vendor_id : hwd.vendor_ids) {
            vendorids += m_vendor_id + " ";
        }

        for (const auto& classID : hwd.class_ids) {
            classids += classID + " ";
        }
    }
    std::string dependencies;
    for (const auto& dependency : config.dependencies) {
        dependencies += dependency + " ";
    }
    std::string conflicts;
    for (const auto& conflict : config.conflicts) {
        conflicts += conflict + " ";
    }

    std::cout << "   NAME:\t" << config.name
              << "\n   ATTACHED:\t" << config.type
              << "\n   VERSION:\t" << config.version
              << "\n   INFO:\t" << (config.info.empty() ? "-" : config.info)
              << "\n   PRIORITY:\t" << config.priority
              << "\n   FREEDRIVER:\t" << std::boolalpha << config.is_freedriver
              << "\n   DEPENDS:\t" << (dependencies.empty() ? "-" : dependencies)
              << "\n   CONFLICTS:\t" << (conflicts.empty() ? "-" : conflicts)
              << "\n   CLASSIDS:\t" << classids
              << "\n   VENDORIDS:\t" << vendorids << "\n\n";
}

void ConsoleWriter::printLine() const {
    std::cout << std::string(80, '-') << '\n';
}

void ConsoleWriter::printDeviceDetails(hw_item hw, FILE* f) const {
    std::unique_ptr<hd_data_t> hd_data{new hd_data_t()};
    auto* hd = hd_list(hd_data.get(), hw, 1, nullptr);

    for (hd_t* iter = hd; iter; iter = iter->next) {
        hd_dump_entry(hd_data.get(), iter, f);
    }

    hd_free_hd_list(hd);
    hd_free_hd_data(hd_data.get());
}

}  // namespace mhwd
