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
#include "vita/string.hpp"

#include <algorithm>   // for any_of
#include <array>       // for array
#include <filesystem>  // for exists, copy, remove_all
#include <stdexcept>   // for runtime_error
#include <string>      // for string
#include <vector>      // for vector

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

#include <fmt/compile.h>
#include <fmt/core.h>

namespace fs = std::filesystem;

namespace mhwd {

namespace {

inline bool is_user_root() noexcept {
    static constexpr auto ROOT_UID = 0;
    return ROOT_UID == getuid();
}

auto check_environment() -> std::vector<std::string> {
    std::vector<std::string> missingDirs;
    if (!fs::exists(consts::CHWD_USB_CONFIG_DIR)) {
        missingDirs.emplace_back(consts::CHWD_USB_CONFIG_DIR);
    }
    if (!fs::exists(consts::CHWD_PCI_CONFIG_DIR)) {
        missingDirs.emplace_back(consts::CHWD_PCI_CONFIG_DIR);
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

bool Mhwd::performTransaction(const profile_t& config, mhwd::transaction_t transaction_type) {
    const auto transaction = Transaction{config, transaction_type, m_arguments.FORCE};
    const auto& status     = performTransaction(transaction);

    switch (status) {
    case mhwd::status_t::SUCCESS:
        break;
    case mhwd::status_t::ERROR_NOT_INSTALLED:
        mhwd::console_writer::print_error("config '{}' is not installed!", config->name);
        break;
    case mhwd::status_t::ERROR_ALREADY_INSTALLED:
        mhwd::console_writer::print_warning("a version of config '{}' is already installed!\nUse -f/--force to force installation...", config->name);
        break;
    case mhwd::status_t::ERROR_NO_MATCH_LOCAL_CONFIG:
        mhwd::console_writer::print_error("passed config does not match with installed config!");
        break;
    case mhwd::status_t::ERROR_SCRIPT_FAILED:
        mhwd::console_writer::print_error("script failed!");
        break;
    case mhwd::status_t::ERROR_SET_DATABASE:
        mhwd::console_writer::print_error("failed to set database!");
        break;
    }

    m_data.updateInstalledConfigData();

    return (mhwd::status_t::SUCCESS == status);
}

auto Mhwd::getInstalledConfig(const std::string_view& config_name, std::string_view config_type) const noexcept -> profile_t {
    // Get the right configs
    const auto& installed_configs = ("USB" == config_type) ? &m_data.installedUSBConfigs : &m_data.installedPCIConfigs;

    auto installed_config = ranges::find_if(*installed_configs,
        [config_name](const auto& temp) {
            return config_name == temp->name;
        });

    if (installed_config != installed_configs->end()) {
        return *installed_config;
    }
    return nullptr;
}

auto Mhwd::getDatabaseConfig(const std::string_view& config_name, std::string_view config_type) const noexcept -> profile_t {
    // Get the right configs
    const auto& allConfigs = ("USB" == config_type) ? &m_data.allUSBConfigs : &m_data.allPCIConfigs;

    auto config = ranges::find_if(*allConfigs,
        [config_name](const auto& temp) {
            return temp->name == config_name;
        });
    if (config != allConfigs->end()) {
        return *config;
    }
    return nullptr;
}

auto Mhwd::getAvailableConfig(const std::string_view& config_name, std::string_view config_type) const noexcept -> profile_t {
    // Get the right devices
    const auto& devices = ("USB" == config_type) ? m_data.USBDevices : m_data.PCIDevices;

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

auto Mhwd::performTransaction(const Transaction& transaction) -> mhwd::status_t {
    // Check if already installed
    auto installed_config{getInstalledConfig(transaction.config->name,
        transaction.config->type)};
    mhwd::status_t status = mhwd::status_t::SUCCESS;

    if ((mhwd::transaction_t::remove == transaction.type)
        || (installed_config != nullptr && transaction.is_reinstall_allowed)) {
        if (nullptr == installed_config) {
            return mhwd::status_t::ERROR_NOT_INSTALLED;
        }
        mhwd::console_writer::print_message(mhwd::message_t::REMOVE_START, installed_config->name);
        status = uninstallConfig(installed_config.get());
        if (mhwd::status_t::SUCCESS != status) {
            return status;
        }
        mhwd::console_writer::print_message(mhwd::message_t::REMOVE_END, installed_config->name);
    }

    if (mhwd::transaction_t::install == transaction.type) {
        // Check if already installed but not allowed to reinstall
        if ((nullptr != installed_config) && !transaction.is_reinstall_allowed) {
            return mhwd::status_t::ERROR_ALREADY_INSTALLED;
        }
        mhwd::console_writer::print_message(mhwd::message_t::INSTALL_START, transaction.config->name);
        status = installConfig(transaction.config);
        if (mhwd::status_t::SUCCESS != status) {
            return status;
        }
        mhwd::console_writer::print_message(mhwd::message_t::INSTALL_END,
            transaction.config->name);
    }
    return status;
}

auto Mhwd::installConfig(const profile_t& config) -> mhwd::status_t {
    if (!runScript(config, mhwd::transaction_t::install)) {
        return mhwd::status_t::ERROR_SCRIPT_FAILED;
    }

    const auto& databaseDir = ("USB" == config->type) ? consts::MHWD_USB_DATABASE_DIR : consts::MHWD_PCI_DATABASE_DIR;
    if (!mhwd::Profile::write_profile_to_file(fmt::format(FMT_COMPILE("{}/{}"), databaseDir, consts::CHWD_CONFIG_FILE), *config)) {
        return mhwd::status_t::ERROR_SET_DATABASE;
    }

    // Installed config vectors have to be updated manual with updateInstalledConfigData(Data*)
    return mhwd::status_t::SUCCESS;
}

auto Mhwd::uninstallConfig(Profile* config) noexcept -> mhwd::status_t {
    auto installed_config{getInstalledConfig(config->name, config->type)};

    // Check if installed
    if (nullptr == installed_config) {
        return mhwd::status_t::ERROR_NOT_INSTALLED;
    }
    // Run script
    if (!runScript(installed_config, mhwd::transaction_t::remove)) {
        return mhwd::status_t::ERROR_SCRIPT_FAILED;
    }

    const auto& databaseDir = ("USB" == config->type) ? consts::MHWD_USB_DATABASE_DIR : consts::MHWD_PCI_DATABASE_DIR;

    std::error_code err_code{};
    fs::remove(fmt::format(FMT_COMPILE("{}/{}"), databaseDir, consts::CHWD_CONFIG_FILE), err_code);
    if (err_code.value() != 0) {
        return mhwd::status_t::ERROR_SET_DATABASE;
    }

    // Installed config vectors have to be updated manual with updateInstalledConfigData(Data*)
    m_data.updateInstalledConfigData();

    return mhwd::status_t::SUCCESS;
}

bool Mhwd::runScript(const profile_t& config, mhwd::transaction_t operation) noexcept {
    auto cmd              = fmt::format(FMT_COMPILE("exec {}"), consts::CHWD_SCRIPT_PATH);
    const auto& conf_path = ("USB" == config->type) ? consts::CHWD_USB_CONFIG_DIR : consts::CHWD_PCI_CONFIG_DIR;

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
    cmd += fmt::format(FMT_COMPILE(" --profile \"{}\""), config->name);
    cmd += fmt::format(FMT_COMPILE(" --path \"{}\""), fmt::format(FMT_COMPILE("{}/{}"), conf_path, consts::CHWD_CONFIG_FILE));

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

    static constexpr auto HEX_BASE = 16;
    for (auto&& dev : devices) {
        auto busID = dev->sysfs_busid;

        if ("PCI" == config->type) {
            const auto& split = Vita::string(busID).replace(".", ":").explode(":");
            const auto& size  = split.size();

            if (size >= 3) {
                // Convert to int to remove leading 0
                busID = fmt::format(FMT_COMPILE("{}:{}:{}"),
                    std::stoi(split[size - 3], nullptr, HEX_BASE),
                    std::stoi(split[size - 2], nullptr, HEX_BASE),
                    std::stoi(split[size - 1], nullptr, HEX_BASE));
            }
        }

        cmd += fmt::format(FMT_COMPILE(" --device \"{}|{}|{}|{}\""), dev->class_id, dev->vendor_id, dev->device_id, busID);
    }

    cmd += " 2>&1";

    const auto& exit_code = std::system(cmd.c_str());
    if (exit_code != 0) {
        return false;
    }
    // Only one database sync is required
    if (mhwd::transaction_t::install == operation) {
        m_data.environment.syncPackageManagerDatabase = false;
    }
    return true;
}

auto Mhwd::tryToParseCmdLineOptions(std::span<char*> args, bool& autoconf_nonfree_driver, std::string& operation, std::string& autoconf_class_id) noexcept(false) -> std::int32_t {
    if (args.size() <= 1) {
        m_arguments.LIST_AVAILABLE = true;
    }
    const auto& proceed_install_option = [&operation](const auto& option, const auto& argument) {
        const std::string_view& device_type{argument};
        if (("pci" != device_type) && ("usb" != device_type)) {
            throw std::runtime_error{fmt::format(FMT_COMPILE("Invalid use of option: {}\n"), option)};
        }
        operation = Vita::string{device_type}.to_upper();
    };
    for (std::size_t nArg = 1; nArg < args.size(); ++nArg) {
        const std::string_view option{args[nArg]};
        if (("-h" == option) || ("--help" == option)) {
            mhwd::console_writer::print_help();
            return 1;
        } else if (("-v" == option) || ("--version" == option)) {
            mhwd::console_writer::print_version(m_version, m_year);
            return 1;
        } else if ("--is_nvidia_card" == option) {
            checkNvidiaCard();
            return 1;
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
            if (nArg >= args.size()) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Too few arguments: {}\n"), option)};
            }
            const std::string_view& device_type{args[nArg - 2]};
            const std::string_view& driver_type{args[nArg - 1]};
            const std::string_view& class_id{args[nArg]};
            if ((("pci" != device_type) && ("usb" != device_type))
                || (("free" != driver_type) && ("nonfree" != driver_type))) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Invalid use of option: {}\n"), option)};
            }
            operation                 = Vita::string{device_type}.to_upper();
            autoconf_nonfree_driver   = ("nonfree" == driver_type);
            autoconf_class_id         = Vita::string{class_id}.to_lower().trim();
            m_arguments.AUTOCONFIGURE = true;
        } else if (("-i" == option) || ("--install" == option)) {
            ++nArg;
            if (nArg >= args.size()) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Too few arguments: {}\n"), option)};
            }
            m_arguments.INSTALL = true;
            proceed_install_option(option, args[nArg]);
        } else if (("-r" == option) || ("--remove" == option)) {
            if ((nArg + 1) >= args.size()) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Too few arguments: {}\n"), option)};
            }
            const std::string_view& device_type{args[++nArg]};
            if (("pci" != device_type) && ("usb" != device_type)) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Invalid device type: {}\n"), device_type)};
            }
            operation          = Vita::string{device_type}.to_upper();
            m_arguments.REMOVE = true;
        } else if ("--pmcachedir" == option) {
            if ((nArg + 1) >= args.size()) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Too few arguments: {}\n"), option)};
            }
            m_data.environment.PMCachePath = Vita::string(args[++nArg]).trim("\"").trim();
        } else if ("--pmconfig" == option) {
            if (nArg + 1 >= args.size()) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Too few arguments: {}\n"), option)};
            }
            m_data.environment.PMConfigPath = Vita::string(args[++nArg]).trim("\"").trim();
        } else if ("--pmroot" == option) {
            if (nArg + 1 >= args.size()) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Too few arguments: {}\n"), option)};
            }
            m_data.environment.PMRootPath = Vita::string(args[++nArg]).trim("\"").trim();
        } else if (m_arguments.INSTALL || m_arguments.REMOVE) {
            const auto& name = Vita::string(args[nArg]).to_lower();
            if (!ranges::any_of(m_configs, [name](auto&& config) { return name == config; })) {
                m_configs.push_back(name);
            }
        } else {
            throw std::runtime_error{fmt::format(FMT_COMPILE("invalid option: {}\n"), args[nArg])};
        }
    }
    if (!m_arguments.SHOW_PCI && !m_arguments.SHOW_USB) {
        m_arguments.SHOW_USB = true;
        m_arguments.SHOW_PCI = true;
    }

    return 0;
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

auto Mhwd::launch(std::span<char*> args) -> std::int32_t {
    std::string operation{};
    bool autoconf_nonfree_driver{false};
    std::string autoconf_class_id{};

    try {
        const std::int32_t cmdline_code = tryToParseCmdLineOptions(args,
            autoconf_nonfree_driver, operation,
            autoconf_class_id);
        /* clang-format off */
        if (cmdline_code == 1) { return 0; }
        /* clang-format on */
        optionsDontInterfereWithEachOther();
    } catch (const std::runtime_error& e) {
        mhwd::console_writer::print_error(e.what());
        mhwd::console_writer::print_help();
        return 1;
    }

    m_data = mhwd::Data::initialize_data();

    const auto& missingDirs = check_environment();
    if (!missingDirs.empty()) {
        mhwd::console_writer::print_error("Following directories do not exist:");
        for (const auto& dir : missingDirs) {
            mhwd::console_writer::print_status(dir);
        }
        return 1;
    }

    // Check for invalid configs
    for (const auto& invalidConfig : m_data.invalidConfigs) {
        mhwd::console_writer::print_warning("config '{}' is invalid!", invalidConfig);
    }

    // > Perform operations:

    // List all configs
    if (m_arguments.LIST_ALL && m_arguments.SHOW_PCI) {
        if (!m_data.allPCIConfigs.empty()) {
            mhwd::console_writer::list_configs(m_data.allPCIConfigs, "All PCI configs:");
        } else {
            mhwd::console_writer::print_warning("No PCI configs found!");
        }
    }
    if (m_arguments.LIST_ALL && m_arguments.SHOW_USB) {
        if (!m_data.allUSBConfigs.empty()) {
            mhwd::console_writer::list_configs(m_data.allUSBConfigs, "All USB configs:");
        } else {
            mhwd::console_writer::print_warning("No USB configs found!");
        }
    }

    // List installed configs
    if (m_arguments.LIST_INSTALLED && m_arguments.SHOW_PCI) {
        if (m_arguments.DETAIL) {
            mhwd::console_writer::printInstalledConfigs("PCI", m_data.installedPCIConfigs);
        } else {
            if (!m_data.installedPCIConfigs.empty()) {
                mhwd::console_writer::list_configs(m_data.installedPCIConfigs, "Installed PCI configs:");
            } else {
                mhwd::console_writer::print_warning("No installed PCI configs!");
            }
        }
    }
    if (m_arguments.LIST_INSTALLED && m_arguments.SHOW_USB) {
        if (m_arguments.DETAIL) {
            mhwd::console_writer::printInstalledConfigs("USB", m_data.installedUSBConfigs);
        } else {
            if (!m_data.installedUSBConfigs.empty()) {
                mhwd::console_writer::list_configs(m_data.installedUSBConfigs, "Installed USB configs:");
            } else {
                mhwd::console_writer::print_warning("No installed USB configs!");
            }
        }
    }

    // List available configs
    if (m_arguments.LIST_AVAILABLE && m_arguments.SHOW_PCI) {
        if (m_arguments.DETAIL) {
            mhwd::console_writer::printAvailableConfigsInDetail("PCI", m_data.PCIDevices);
        } else {
            for (auto&& PCIdevice : m_data.PCIDevices) {
                if (!PCIdevice->available_configs.empty()) {
                    mhwd::console_writer::list_configs(PCIdevice->available_configs,
                        fmt::format(FMT_COMPILE("{} ({}:{}:{}) {} {}:"), PCIdevice->sysfs_busid, PCIdevice->class_id,
                            PCIdevice->vendor_id, PCIdevice->device_id,
                            PCIdevice->class_name, PCIdevice->vendor_name));
                }
            }
        }
    }

    if (m_arguments.LIST_AVAILABLE && m_arguments.SHOW_USB) {
        if (m_arguments.DETAIL) {
            mhwd::console_writer::printAvailableConfigsInDetail("USB", m_data.USBDevices);
        } else {
            for (auto&& USBdevice : m_data.USBDevices) {
                if (!USBdevice->available_configs.empty()) {
                    mhwd::console_writer::list_configs(USBdevice->available_configs,
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
            mhwd::console_writer::printDeviceDetails(hw_pci);
        } else {
            mhwd::console_writer::list_devices(m_data.PCIDevices, "PCI");
        }
    }
    if (m_arguments.LIST_HARDWARE && m_arguments.SHOW_USB) {
        if (m_arguments.DETAIL) {
            mhwd::console_writer::printDeviceDetails(hw_usb);
        } else {
            mhwd::console_writer::list_devices(m_data.USBDevices, "USB");
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
            profile_t config;

            for (auto&& available_config : device->available_configs) {
                if (autoconf_nonfree_driver || !available_config->is_nonfree) {
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
                mhwd::console_writer::print_warning("No config found for device: {}", device_info);
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
                mhwd::console_writer::print_status("Skipping already installed config '{}' for device: {}", config->name, device_info);
            } else {
                mhwd::console_writer::print_status("Using config '{}' for device: {}", config->name, device_info);
            }

            const bool config_exists = ranges::find(m_configs, config->name) != m_configs.cend();
            if (!config_exists && !skip) {
                m_configs.push_back(config->name);
            }
        }

        if (!found_device) {
            mhwd::console_writer::print_warning("No device of class {} found!", autoconf_class_id);
        } else if (!m_configs.empty()) {
            m_arguments.INSTALL = true;
        }
    }

    // Transaction
    /* clang-format off */
    if (!(m_arguments.INSTALL || m_arguments.REMOVE)) { return 0; }
    /* clang-format on */
    if (!is_user_root()) {
        mhwd::console_writer::print_error("You cannot perform this operation unless you are root!");
        return 1;
    }
    for (auto&& config_name : m_configs) {
        if (m_arguments.INSTALL) {
            m_config = getAvailableConfig(config_name, operation);
            if (m_config == nullptr) {
                m_config = getDatabaseConfig(config_name, operation);
                if (m_config == nullptr) {
                    mhwd::console_writer::print_error("config '{}' does not exist!", config_name);
                    return 1;
                }
                mhwd::console_writer::print_warning("no matching device for config '{}' found!", config_name);
            }

            if (!performTransaction(m_config, mhwd::transaction_t::install)) {
                return 1;
            }
        } else if (m_arguments.REMOVE) {
            m_config = getInstalledConfig(config_name, operation);

            if (nullptr == m_config) {
                mhwd::console_writer::print_error("config '{}' is not installed!", config_name);
                return 1;
            } else if (!performTransaction(m_config, mhwd::transaction_t::remove)) {
                return 1;
            }
        }
    }
    return 0;
}

void Mhwd::checkNvidiaCard() noexcept {
    if (!fs::exists("/var/lib/mhwd/ids/pci/nvidia.ids")) {
        fmt::print("No nvidia ids found!\n");
        return;
    }

    m_data = mhwd::Data::initialize_data();
    for (auto&& PCIdevice : m_data.PCIDevices) {
        /* clang-format off */
        if (PCIdevice->available_configs.empty()) { continue; }
        /* clang-format on */

        if (PCIdevice->vendor_id == "10de") {
            fmt::print("NVIDIA card found!\n");
            return;
        }
    }
}

}  // namespace mhwd
