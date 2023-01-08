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

#include "mhwd.hpp"

#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

#include <algorithm>
#include <array>
#include <cctype>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <memory>
#include <stdexcept>
#include <string>
#include <vector>

#if defined(__clang__)
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wold-style-cast"
#pragma clang diagnostic ignored "-Wimplicit-int-conversion"

#include <range/v3/algorithm/any_of.hpp>
#include <range/v3/algorithm/find.hpp>
#include <range/v3/algorithm/find_if.hpp>

#pragma clang diagnostic pop
#else
#include <ranges>
namespace ranges = std::ranges;
#endif

#include "vita/string.hpp"

#include <fmt/compile.h>
#include <fmt/core.h>

namespace fs = std::filesystem;

namespace mhwd {

namespace {
    std::string gatherConfigContent(const list_of_configs_t& configuration) {
        std::string config;
        for (auto&& c : configuration) {
            config += fmt::format(FMT_COMPILE(" {}"), c->name);
        }
        return config;
    }

    bool is_user_root() noexcept {
        static constexpr auto ROOT_UID = 0;
        return ROOT_UID == getuid();
    }

    std::vector<std::string> checkEnvironment() {
        std::vector<std::string> missingDirs;
        if (!fs::exists(consts::MHWD_USB_CONFIG_DIR)) {
            missingDirs.emplace_back(consts::MHWD_USB_CONFIG_DIR);
        }
        if (!fs::exists(consts::MHWD_PCI_CONFIG_DIR)) {
            missingDirs.emplace_back(consts::MHWD_PCI_CONFIG_DIR);
        }
        if (!fs::exists(consts::MHWD_USB_DATABASE_DIR)) {
            missingDirs.emplace_back(consts::MHWD_USB_DATABASE_DIR);
        }
        if (!fs::exists(consts::MHWD_PCI_DATABASE_DIR)) {
            missingDirs.emplace_back(consts::MHWD_PCI_DATABASE_DIR);
        }

        return missingDirs;
    }
}  // namespace

bool Mhwd::performTransaction(const config_t& config, mhwd::transaction_t transactionType) {
    Transaction transaction(m_data, config, transactionType, m_arguments.FORCE);

    // Print things to do
    if (mhwd::transaction_t::install == transactionType) {
        // Print conflicts
        if (!transaction.conflicted_configs.empty()) {
            m_console_writer.print_error("config '{}' conflicts with config(s):{}", config->name, gatherConfigContent(transaction.conflicted_configs));
            return false;
        }

        // Print dependencies
        else if (!transaction.dependency_configs.empty()) {
            m_console_writer.print_status("Dependencies to install: {}", gatherConfigContent(transaction.dependency_configs));
        }
    } else if (mhwd::transaction_t::remove == transactionType) {
        // Print requirements
        if (!transaction.configs_requirements.empty()) {
            m_console_writer.print_error("config '{}' is required by config(s):{}", config->name, gatherConfigContent(transaction.configs_requirements));
            return false;
        }
    }

    mhwd::status_t status = performTransaction(transaction);

    switch (status) {
    case mhwd::status_t::SUCCESS:
        break;
    case mhwd::status_t::ERROR_CONFLICTS:
        m_console_writer.print_error("config '{}' conflicts with installed config(s)!", config->name);
        break;
    case mhwd::status_t::ERROR_REQUIREMENTS:
        m_console_writer.print_error("config '{}' is required by installed config(s)!", config->name);
        break;
    case mhwd::status_t::ERROR_NOT_INSTALLED:
        m_console_writer.print_error("config '{}' is not installed!", config->name);
        break;
    case mhwd::status_t::ERROR_ALREADY_INSTALLED:
        m_console_writer.print_warning("a version of config '{}' is already installed!\nUse -f/--force to force installation...", config->name);
        break;
    case mhwd::status_t::ERROR_NO_MATCH_LOCAL_CONFIG:
        m_console_writer.print_error("passed config does not match with installed config!");
        break;
    case mhwd::status_t::ERROR_SCRIPT_FAILED:
        m_console_writer.print_error("script failed!");
        break;
    case mhwd::status_t::ERROR_SET_DATABASE:
        m_console_writer.print_error("failed to set database!");
        break;
    }

    m_data.updateInstalledConfigData();

    return (mhwd::status_t::SUCCESS == status);
}

config_t Mhwd::getInstalledConfig(const std::string_view& config_name, const std::string_view& configType) const noexcept {
    // Get the right configs
    const auto& installed_configs = ("USB" == configType) ? &m_data.installedUSBConfigs : &m_data.installedPCIConfigs;

    auto installed_config = ranges::find_if(*installed_configs,
        [config_name](const auto& temp) {
            return config_name == temp->name;
        });

    if (installed_config != installed_configs->end()) {
        return *installed_config;
    }
    return nullptr;
}

config_t Mhwd::getDatabaseConfig(const std::string_view& config_name, const std::string_view& configType) const noexcept {
    // Get the right configs
    const auto& allConfigs = ("USB" == configType) ? &m_data.allUSBConfigs : &m_data.allPCIConfigs;

    auto config = ranges::find_if(*allConfigs,
        [config_name](const auto& temp) {
            return temp->name == config_name;
        });
    if (config != allConfigs->end()) {
        return *config;
    }
    return nullptr;
}

config_t Mhwd::getAvailableConfig(const std::string_view& config_name, const std::string_view& configType) const noexcept {
    // Get the right devices
    const auto& devices = ("USB" == configType) ? m_data.USBDevices : m_data.PCIDevices;

    for (auto&& device : devices) {
        if (device->available_configs.empty()) {
            continue;
        }
        auto& available_configs = device->available_configs;
        auto available_config   = ranges::find_if(available_configs,
              [config_name](const auto& temp) {
                return temp->name == config_name;
              });
        if (available_config != available_configs.end()) {
            return *available_config;
        }
    }
    return nullptr;
}

mhwd::status_t Mhwd::performTransaction(const Transaction& transaction) {
    if ((mhwd::transaction_t::install == transaction.type) && !transaction.conflicted_configs.empty()) {
        return mhwd::status_t::ERROR_CONFLICTS;
    } else if ((mhwd::transaction_t::remove == transaction.type)
        && !transaction.configs_requirements.empty()) {
        return mhwd::status_t::ERROR_REQUIREMENTS;
    }
    // Check if already installed
    auto installed_config{getInstalledConfig(transaction.config->name,
        transaction.config->type)};
    mhwd::status_t status = mhwd::status_t::SUCCESS;

    if ((mhwd::transaction_t::remove == transaction.type)
        || (installed_config != nullptr && transaction.is_reinstall_allowed)) {
        if (nullptr == installed_config) {
            return mhwd::status_t::ERROR_NOT_INSTALLED;
        }
        m_console_writer.print_message(mhwd::message_t::REMOVE_START, installed_config->name);
        status = uninstallConfig(installed_config.get());
        if (mhwd::status_t::SUCCESS != status) {
            return status;
        }
        m_console_writer.print_message(mhwd::message_t::REMOVE_END, installed_config->name);
    }

    if (mhwd::transaction_t::install == transaction.type) {
        // Check if already installed but not allowed to reinstall
        if ((nullptr != installed_config) && !transaction.is_reinstall_allowed) {
            return mhwd::status_t::ERROR_ALREADY_INSTALLED;
        }
        // Install all dependencies first
        for (auto&& dependencyConfig = transaction.dependency_configs.end() - 1;
             dependencyConfig != transaction.dependency_configs.begin() - 1;
             --dependencyConfig) {
            m_console_writer.print_message(mhwd::message_t::INSTALLDEPENDENCY_START,
                (*dependencyConfig)->name);
            status = installConfig((*dependencyConfig));
            if (mhwd::status_t::SUCCESS != status) {
                return status;
            }
            m_console_writer.print_message(mhwd::message_t::INSTALLDEPENDENCY_END,
                (*dependencyConfig)->name);
        }

        m_console_writer.print_message(mhwd::message_t::INSTALL_START, transaction.config->name);
        status = installConfig(transaction.config);
        if (mhwd::status_t::SUCCESS != status) {
            return status;
        }
        m_console_writer.print_message(mhwd::message_t::INSTALL_END,
            transaction.config->name);
    }
    return status;
}

mhwd::status_t Mhwd::installConfig(const config_t& config) {
    const auto& databaseDir = ("USB" == config->type) ? consts::MHWD_USB_DATABASE_DIR : consts::MHWD_PCI_DATABASE_DIR;
    if (!runScript(config, mhwd::transaction_t::install)) {
        return mhwd::status_t::ERROR_SCRIPT_FAILED;
    }

    std::error_code ec;
    fs::copy(config->base_path, fmt::format(FMT_COMPILE("{}/{}"), databaseDir, config->name), fs::copy_options::recursive, ec);
    if (ec.value() != 0) {
        return mhwd::status_t::ERROR_SET_DATABASE;
    }

    // Installed config vectors have to be updated manual with updateInstalledConfigData(Data*)
    return mhwd::status_t::SUCCESS;
}

mhwd::status_t Mhwd::uninstallConfig(Config* config) noexcept {
    auto installed_config{getInstalledConfig(config->name, config->type)};

    // Check if installed
    if (nullptr == installed_config) {
        return mhwd::status_t::ERROR_NOT_INSTALLED;
    } else if (installed_config->base_path != config->base_path) {
        return mhwd::status_t::ERROR_NO_MATCH_LOCAL_CONFIG;
    }
    // Run script
    if (!runScript(installed_config, mhwd::transaction_t::remove)) {
        return mhwd::status_t::ERROR_SCRIPT_FAILED;
    }

    std::error_code ec;
    fs::remove_all(installed_config->base_path, ec);
    if (ec.value() != 0) {
        return mhwd::status_t::ERROR_SET_DATABASE;
    }

    // Installed config vectors have to be updated manual with updateInstalledConfigData(Data*)
    m_data.updateInstalledConfigData();

    return mhwd::status_t::SUCCESS;
}

bool Mhwd::runScript(const config_t& config, mhwd::transaction_t operation) noexcept {
    auto cmd = fmt::format(FMT_COMPILE("exec {}"), consts::MHWD_SCRIPT_PATH);

    if (mhwd::transaction_t::remove == operation) {
        cmd += " --remove";
    } else {
        cmd += " --install";
    }

    if (m_data.environment.syncPackageManagerDatabase) {
        cmd += " --sync";
    }

    cmd += fmt::format(FMT_COMPILE(" --cachedir \"{}\""), m_data.environment.PMCachePath);
    cmd += fmt::format(FMT_COMPILE(" --pmconfig \"{}\""), m_data.environment.PMConfigPath);
    cmd += fmt::format(FMT_COMPILE(" --pmroot \"{}\""), m_data.environment.PMRootPath);
    cmd += fmt::format(FMT_COMPILE(" --config \"{}\""), config->config_path);

    // Set all config devices as argument
    list_of_devices_t found_devices;
    list_of_devices_t devices;
    m_data.getAllDevicesOfConfig(config, found_devices);

    for (auto&& found_device : found_devices) {
        // Check if already in list
        const bool found = ranges::any_of(devices,
            [&found_device](auto&& dev) { return (found_device->sysfs_busid == dev->sysfs_busid)
                                              && (found_device->sysfs_id == dev->sysfs_id); });

        if (!found) {
            devices.push_back(found_device);
        }
    }

    for (auto&& dev : devices) {
        auto busID = dev->sysfs_busid;

        if ("PCI" == config->type) {
            const auto& split = Vita::string(busID).replace(".", ":").explode(":");
            const auto& size  = split.size();

            if (size >= 3) {
                // Convert to int to remove leading 0
                busID = fmt::format(FMT_COMPILE("{}:{}:{}"),
                    Vita::string::toStr<int>(std::stoi(split[size - 3], nullptr, 16)),
                    Vita::string::toStr<int>(std::stoi(split[size - 2], nullptr, 16)),
                    Vita::string::toStr<int>(std::stoi(split[size - 1], nullptr, 16)));
            }
        }

        cmd += fmt::format(FMT_COMPILE(" --device \"{}|{}|{}|{}\""), dev->class_id, dev->vendor_id, dev->device_id, busID);
    }

    cmd += " 2>&1";

    auto* in = popen(cmd.c_str(), "r");
    if (!in) {
        return false;
    }
    std::array<char, 512> buf{};
    while (fgets(buf.data(), buf.size(), in) != nullptr) {
        m_console_writer.print_message(mhwd::message_t::CONSOLE_OUTPUT, buf.data());
    }

    int stat = pclose(in);
    if (WEXITSTATUS(stat) != 0) {
        return false;
    }
    // Only one database sync is required
    if (mhwd::transaction_t::install == operation) {
        m_data.environment.syncPackageManagerDatabase = false;
    }
    return true;
}

void Mhwd::tryToParseCmdLineOptions(int argc, char** argv, bool& autoconf_nonfree_driver, std::string& operation, std::string& autoconf_class_id) noexcept(false) {
    if (argc <= 1) {
        m_arguments.LIST_AVAILABLE = true;
    }
    const auto& proceed_install_option = [&operation](const auto& option, const auto& argument) {
        const std::string_view& device_type{argument};
        if (("pci" != device_type) && ("usb" != device_type)) {
            throw std::runtime_error{fmt::format(FMT_COMPILE("Invalid use of option: {}\n"), option)};
        }
        operation = Vita::string{device_type}.toUpper();
    };
    for (int nArg = 1; nArg < argc; ++nArg) {
        const std::string_view option{argv[nArg]};
        if (("-h" == option) || ("--help" == option)) {
            mhwd::ConsoleWriter::print_help();
        } else if (("-v" == option) || ("--version" == option)) {
            mhwd::ConsoleWriter::print_version(m_version, m_year);
        } else if (("-f" == option) || ("--force" == option)) {
            m_arguments.FORCE = true;
        } else if (("-d" == option) || ("--detail" == option)) {
            m_arguments.DETAIL = true;
        } else if (("-la" == option) || ("--listall" == option)) {
            m_arguments.LIST_ALL = true;
        } else if (("-li" == option) || ("--listinstalled" == option)) {
            m_arguments.LIST_INSTALLED = true;
        } else if (("-l" == option) || ("--list" == option)) {
            m_arguments.LIST_AVAILABLE = true;
        } else if (("-lh" == option) || ("--listhardware" == option)) {
            m_arguments.LIST_HARDWARE = true;
        } else if ("--pci" == option) {
            m_arguments.SHOW_PCI = true;
        } else if ("--usb" == option) {
            m_arguments.SHOW_USB = true;
        } else if (("-a" == option) || ("--auto" == option)) {
            nArg += 3;
            if (nArg >= argc) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Too few arguments: {}\n"), option)};
            }
            const std::string_view& device_type{argv[nArg - 2]};
            const std::string_view& driver_type{argv[nArg - 1]};
            const std::string_view& class_id{argv[nArg]};
            if ((("pci" != device_type) && ("usb" != device_type))
                || (("free" != driver_type) && ("nonfree" != driver_type))) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Invalid use of option: {}\n"), option)};
            }
            operation                 = Vita::string{device_type}.toUpper();
            autoconf_nonfree_driver   = ("nonfree" == driver_type);
            autoconf_class_id         = Vita::string{class_id}.toLower().trim();
            m_arguments.AUTOCONFIGURE = true;
        } else if (("-ic" == option) || ("--installcustom" == option)) {
            ++nArg;
            if (nArg >= argc) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Too few arguments: {}\n"), option)};
            }
            m_arguments.CUSTOM_INSTALL = true;
            proceed_install_option(option, argv[nArg]);
        } else if (("-i" == option) || ("--install" == option)) {
            ++nArg;
            if (nArg >= argc) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Too few arguments: {}\n"), option)};
            }
            m_arguments.INSTALL = true;
            proceed_install_option(option, argv[nArg]);
        } else if (("-r" == option) || ("--remove" == option)) {
            if ((nArg + 1) >= argc) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Too few arguments: {}\n"), option)};
            }
            const std::string_view& device_type{argv[++nArg]};
            if (("pci" != device_type) && ("usb" != device_type)) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Invalid device type: {}\n"), device_type)};
            }
            operation          = Vita::string{device_type}.toUpper();
            m_arguments.REMOVE = true;
        } else if ("--pmcachedir" == option) {
            if ((nArg + 1) >= argc) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Too few arguments: {}\n"), option)};
            }
            m_data.environment.PMCachePath = Vita::string(argv[++nArg]).trim("\"").trim();
        } else if ("--pmconfig" == option) {
            if (nArg + 1 >= argc) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Too few arguments: {}\n"), option)};
            }
            m_data.environment.PMConfigPath = Vita::string(argv[++nArg]).trim("\"").trim();
        } else if ("--pmroot" == option) {
            if (nArg + 1 >= argc) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Too few arguments: {}\n"), option)};
            }
            m_data.environment.PMRootPath = Vita::string(argv[++nArg]).trim("\"").trim();
        } else if (m_arguments.INSTALL || m_arguments.REMOVE) {
            const auto& name = (m_arguments.CUSTOM_INSTALL) ? argv[nArg] : Vita::string(argv[nArg]).toLower();
            if (!ranges::any_of(m_configs, [name](auto&& config) { return name == config; })) {
                m_configs.push_back(name);
            }
        } else {
            throw std::runtime_error{fmt::format(FMT_COMPILE("invalid option: {}\n"), argv[nArg])};
        }
    }
    if (!m_arguments.SHOW_PCI && !m_arguments.SHOW_USB) {
        m_arguments.SHOW_USB = true;
        m_arguments.SHOW_PCI = true;
    }
}

void Mhwd::optionsDontInterfereWithEachOther() const noexcept(false) {
    if (m_arguments.INSTALL && m_arguments.REMOVE) {
        throw std::runtime_error{"install and remove options can only be used separately!\n"};
    } else if ((m_arguments.INSTALL || m_arguments.REMOVE) && m_arguments.AUTOCONFIGURE) {
        throw std::runtime_error{"auto option can't be combined with install and remove options!\n"};
    } else if ((m_arguments.REMOVE || m_arguments.INSTALL) && m_configs.empty()) {
        throw std::runtime_error{"nothing to do?!\n"};
    }
}

int Mhwd::launch(int argc, char** argv) {
    std::string operation;
    bool autoconf_nonfree_driver = false;
    std::string autoconf_class_id;

    try {
        tryToParseCmdLineOptions(argc, argv, autoconf_nonfree_driver, operation,
            autoconf_class_id);
        optionsDontInterfereWithEachOther();
    } catch (const std::runtime_error& e) {
        m_console_writer.print_error(e.what());
        m_console_writer.print_help();
        return 1;
    }

    std::vector<std::string> missingDirs{checkEnvironment()};
    if (!missingDirs.empty()) {
        m_console_writer.print_error("Following directories do not exist:");
        for (const auto& dir : missingDirs) {
            m_console_writer.print_status(dir);
        }
        return 1;
    }

    // Check for invalid configs
    for (auto&& invalidConfig : m_data.invalidConfigs) {
        m_console_writer.print_warning("config '{}' is invalid!", invalidConfig->config_path);
    }

    // > Perform operations:

    // List all configs
    if (m_arguments.LIST_ALL && m_arguments.SHOW_PCI) {
        if (!m_data.allPCIConfigs.empty()) {
            m_console_writer.list_configs(m_data.allPCIConfigs, "All PCI configs:");
        } else {
            m_console_writer.print_warning("No PCI configs found!");
        }
    }
    if (m_arguments.LIST_ALL && m_arguments.SHOW_USB) {
        if (!m_data.allUSBConfigs.empty()) {
            m_console_writer.list_configs(m_data.allUSBConfigs, "All USB configs:");
        } else {
            m_console_writer.print_warning("No USB configs found!");
        }
    }

    // List installed configs
    if (m_arguments.LIST_INSTALLED && m_arguments.SHOW_PCI) {
        if (m_arguments.DETAIL) {
            m_console_writer.printInstalledConfigs("PCI", m_data.installedPCIConfigs);
        } else {
            if (!m_data.installedPCIConfigs.empty()) {
                m_console_writer.list_configs(m_data.installedPCIConfigs, "Installed PCI configs:");
            } else {
                m_console_writer.print_warning("No installed PCI configs!");
            }
        }
    }
    if (m_arguments.LIST_INSTALLED && m_arguments.SHOW_USB) {
        if (m_arguments.DETAIL) {
            m_console_writer.printInstalledConfigs("USB", m_data.installedUSBConfigs);
        } else {
            if (!m_data.installedUSBConfigs.empty()) {
                m_console_writer.list_configs(m_data.installedUSBConfigs, "Installed USB configs:");
            } else {
                m_console_writer.print_warning("No installed USB configs!");
            }
        }
    }

    // List available configs
    if (m_arguments.LIST_AVAILABLE && m_arguments.SHOW_PCI) {
        if (m_arguments.DETAIL) {
            m_console_writer.printAvailableConfigsInDetail("PCI", m_data.PCIDevices);
        } else {
            for (auto&& PCIdevice : m_data.PCIDevices) {
                if (!PCIdevice->available_configs.empty()) {
                    m_console_writer.list_configs(PCIdevice->available_configs,
                        fmt::format(FMT_COMPILE("{} ({}:{}:{}) {} {}:"), PCIdevice->sysfs_busid, PCIdevice->class_id,
                            PCIdevice->vendor_id, PCIdevice->device_id,
                            PCIdevice->class_name, PCIdevice->vendor_name));
                }
            }
        }
    }

    if (m_arguments.LIST_AVAILABLE && m_arguments.SHOW_USB) {
        if (m_arguments.DETAIL) {
            m_console_writer.printAvailableConfigsInDetail("USB", m_data.USBDevices);
        } else {
            for (auto&& USBdevice : m_data.USBDevices) {
                if (!USBdevice->available_configs.empty()) {
                    m_console_writer.list_configs(USBdevice->available_configs,
                        fmt::format(FMT_COMPILE("{} ({}:{}:{}) {} {}:"), USBdevice->sysfs_busid, USBdevice->class_id,
                            USBdevice->vendor_id, USBdevice->device_id,
                            USBdevice->class_name, USBdevice->vendor_name));
                }
            }
        }
    }

    // List hardware information
    if (m_arguments.LIST_HARDWARE && m_arguments.SHOW_PCI) {
        if (m_arguments.DETAIL) {
            mhwd::ConsoleWriter::printDeviceDetails(hw_pci);
        } else {
            m_console_writer.list_devices(m_data.PCIDevices, "PCI");
        }
    }
    if (m_arguments.LIST_HARDWARE && m_arguments.SHOW_USB) {
        if (m_arguments.DETAIL) {
            mhwd::ConsoleWriter::printDeviceDetails(hw_usb);
        } else {
            m_console_writer.list_devices(m_data.USBDevices, "USB");
        }
    }

    // Auto configuration
    if (m_arguments.AUTOCONFIGURE) {
        const auto& devices           = ("USB" == operation) ? m_data.USBDevices : m_data.PCIDevices;
        const auto& installed_configs = ("USB" == operation) ? m_data.installedUSBConfigs : m_data.installedPCIConfigs;

        bool found_device = false;
        for (auto&& device : devices) {
            if (device->class_id != autoconf_class_id) {
                continue;
            }
            found_device = true;
            config_t config;

            for (auto&& available_config : device->available_configs) {
                // Never auto-install drivers with priority 0 (vesa)
                if (available_config->priority == 0) {
                    continue;
                }
                if (autoconf_nonfree_driver || available_config->is_freedriver) {
                    config = available_config;
                    break;
                }
            }

            const auto& device_info = fmt::format(FMT_COMPILE("{} ({}:{}:{}) {} {} {}"),
                device->sysfs_busid,
                device->class_id, device->vendor_id,
                device->device_id, device->class_name,
                device->vendor_name, device->device_name);

            if (nullptr == config) {
                m_console_writer.print_warning("No config found for device: {}", device_info);
                continue;
            }
            // If force is not set then skip found config
            bool skip = false;
            if (!m_arguments.FORCE) {
                skip = ranges::find_if(installed_configs,
                           [&config](const auto& temp) -> bool {
                               return temp->name == config->name;
                           })
                    != installed_configs.end();
            }
            // Print found config
            if (skip) {
                m_console_writer.print_status("Skipping already installed config '{}' for device: {}", config->name, device_info);
            } else {
                m_console_writer.print_status("Using config '{}' for device: {}", config->name, device_info);
            }

            const bool config_exists = ranges::find(m_configs, config->name) != m_configs.cend();
            if (!config_exists && !skip) {
                m_configs.push_back(config->name);
            }
        }

        if (!found_device) {
            m_console_writer.print_warning("No device of class {} found!", autoconf_class_id);
        } else if (!m_configs.empty()) {
            m_arguments.INSTALL = true;
        }
    }

    // Transaction
    /* clang-format off */
    if (!(m_arguments.INSTALL || m_arguments.REMOVE)) { return 0; }
    /* clang-format on */
    if (!is_user_root()) {
        m_console_writer.print_error("You cannot perform this operation unless you are root!");
        return 1;
    }
    for (auto&& config_name : m_configs) {
        if (m_arguments.CUSTOM_INSTALL) {
            // Custom install -> get configs
            const auto& filepath = fmt::format(FMT_COMPILE("{}/MHWDCONFIG"), config_name);

            if (!fs::exists(filepath)) {
                m_console_writer.print_error("custom config '{}' does not exist!", filepath);
                return 1;
            } else if (!fs::is_regular_file(filepath)) {
                m_console_writer.print_error("custom config '{}' is invalid!", filepath);
                return 1;
            }
            m_config.reset(new Config(filepath, operation));
            if (!m_config->read_file(filepath)) {
                m_console_writer.print_error("failed to read custom config '{}'!", filepath);
                return 1;
            } else if (!performTransaction(m_config, mhwd::transaction_t::install)) {
                return 1;
            }
        } else if (m_arguments.INSTALL) {
            m_config = getAvailableConfig(config_name, operation);
            if (m_config == nullptr) {
                m_config = getDatabaseConfig(config_name, operation);
                if (m_config == nullptr) {
                    m_console_writer.print_error("config '{}' does not exist!", config_name);
                    return 1;
                }
                m_console_writer.print_warning("no matching device for config '{}' found!", config_name);
            }

            if (!performTransaction(m_config, mhwd::transaction_t::install)) {
                return 1;
            }
        } else if (m_arguments.REMOVE) {
            m_config = getInstalledConfig(config_name, operation);

            if (nullptr == m_config) {
                m_console_writer.print_error("config '{}' is not installed!", config_name);
                return 1;
            } else if (!performTransaction(m_config, mhwd::transaction_t::remove)) {
                return 1;
            }
        }
    }
    return 0;
}

}  // namespace mhwd
