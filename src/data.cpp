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

#include "data.hpp"

#include <fnmatch.h>

#include <filesystem>  // for recursive_directory_iterator
#include <iomanip>     // for setw, setfill
#include <string>      // for string
#include <vector>      // for vector

#if defined(__clang__)
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wold-style-cast"
#pragma clang diagnostic ignored "-Wimplicit-int-conversion"

#include <range/v3/algorithm/find_if.hpp>

#pragma clang diagnostic pop
#else
#include <ranges>
namespace ranges = std::ranges;
#endif

namespace fs = std::filesystem;

namespace mhwd {

namespace {
[[nodiscard]] auto get_recursive_directory_listing(const std::string_view& directory_path, const std::string_view& only_filename) noexcept -> std::vector<std::string> {
    std::vector<std::string> file_list{};
    for (const auto& dir_entry : fs::recursive_directory_iterator(directory_path)) {
        const auto& entry_path     = dir_entry.path();
        const auto& entry_filename = entry_path.filename().c_str();

        const auto& filestatus = fs::status(entry_path);
        if (fs::is_regular_file(filestatus) && (only_filename.empty() || (only_filename == entry_filename))) {
            file_list.push_back(entry_path);
        }
    }

    return file_list;
}

void getAllDevicesOfConfig(const list_of_devices_t& devices, const config_t& config, list_of_devices_t& found_devices) noexcept {
    found_devices.clear();

    for (auto&& hwdID : config->hwd_ids) {
        bool found_device{};
        // Check all devices
        for (auto&& i_device : devices) {
            // Check class ids
            bool found = ranges::find_if(hwdID.class_ids, [i_device](const std::string& classID) {
                return fnmatch(classID.c_str(), i_device->class_id.c_str(), FNM_CASEFOLD) == 0;
            }) != hwdID.class_ids.end();

            if (found) {
                // Check blacklisted class ids
                found = ranges::find_if(hwdID.blacklisted_class_ids, [i_device](const std::string& blacklistedClassID) {
                    return fnmatch(blacklistedClassID.c_str(), i_device->class_id.c_str(), FNM_CASEFOLD) == 0;
                }) != hwdID.blacklisted_class_ids.end();

                if (!found) {
                    // Check vendor ids
                    found = ranges::find_if(hwdID.vendor_ids, [i_device](const std::string& vendorID) {
                        return fnmatch(vendorID.c_str(), i_device->vendor_id.c_str(), FNM_CASEFOLD) == 0;
                    }) != hwdID.vendor_ids.end();

                    if (found) {
                        // Check blacklisted vendor ids
                        found = ranges::find_if(hwdID.blacklisted_vendor_ids, [i_device](const std::string& blacklistedVendorID) {
                            return fnmatch(blacklistedVendorID.c_str(), i_device->vendor_id.c_str(), FNM_CASEFOLD) == 0;
                        }) != hwdID.blacklisted_vendor_ids.end();

                        if (!found) {
                            // Check device ids
                            found = ranges::find_if(hwdID.device_ids, [i_device](const std::string& deviceID) {
                                return fnmatch(deviceID.c_str(), i_device->device_id.c_str(), FNM_CASEFOLD) == 0;
                            }) != hwdID.device_ids.end();

                            if (found) {
                                // Check blacklisted device ids
                                found = ranges::find_if(hwdID.blacklisted_device_ids, [i_device](const std::string& blacklistedDeviceID) {
                                    return fnmatch(blacklistedDeviceID.c_str(), i_device->device_id.c_str(), FNM_CASEFOLD) == 0;
                                }) != hwdID.blacklisted_device_ids.end();
                                if (!found) {
                                    found_device = true;
                                    found_devices.push_back(i_device);
                                }
                            }
                        }
                    }
                }
            }
        }

        if (!found_device) {
            found_devices.clear();
            return;
        }
    }
}

void addConfigSorted(list_of_configs_t& configs,
    const config_t& newConfig) noexcept {
    const bool found = ranges::find_if(configs.begin(), configs.end(),
                           [&newConfig](const config_t& config) {
                               return newConfig->name == config->name;
                           })
        != configs.end();

    if (!found) {
        for (size_t i = 0; i < configs.size(); ++i) {
            auto&& config = configs[i];
            if (newConfig->priority > config->priority) {
                configs.insert(configs.begin() + static_cast<std::int64_t>(i), newConfig);
                return;
            }
        }
        configs.push_back(newConfig);
    }
}

void setMatchingConfig(const config_t& config,
    const list_of_devices_t& devices, bool set_as_installed) noexcept {
    list_of_devices_t found_devices;
    ::mhwd::getAllDevicesOfConfig(devices, config, found_devices);

    // Set config to all matching devices
    for (auto& found_device : found_devices) {
        auto& to_be_added = (set_as_installed) ? found_device->installed_configs : found_device->available_configs;
        addConfigSorted(to_be_added, config);
    }
}

void setMatchingConfigs(const list_of_devices_t& devices,
    const list_of_configs_t& configs, bool set_as_installed) noexcept {
    for (const auto& config : configs) {
        setMatchingConfig(config, devices, set_as_installed);
    }
}

void fillDevices(hw_item item, list_of_devices_t& devices) noexcept {
    const auto& from_hex = [](std::uint16_t hex_number, int fill) noexcept -> Vita::string {
        std::stringstream stream;
        stream << std::hex << std::setfill('0') << std::setw(fill) << hex_number;
        return Vita::string{stream.str()};
    };

    const auto& safe_str = [](auto&& data) noexcept -> std::string_view {
        return (data == nullptr) ? "" : data;
    };

    // Get the hardware devices
    auto hd_data      = std::make_unique<hd_data_t>();
    hd_t* hd_list_obj = hd_list(hd_data.get(), item, 1, nullptr);

    for (hd_t* iter = hd_list_obj; iter != nullptr; iter = iter->next) {
        auto device         = std::make_unique<Device>();
        device->type        = (item == hw_usb) ? "USB" : "PCI";
        device->class_id    = from_hex(static_cast<uint16_t>(iter->base_class.id), 2) + from_hex(static_cast<uint16_t>(iter->sub_class.id), 2).to_lower();
        device->vendor_id   = from_hex(static_cast<uint16_t>(iter->vendor.id), 4).to_lower();
        device->device_id   = from_hex(static_cast<uint16_t>(iter->device.id), 4).to_lower();
        device->class_name  = safe_str(iter->base_class.name);
        device->vendor_name = safe_str(iter->vendor.name);
        device->device_name = safe_str(iter->device.name);
        device->sysfs_busid = safe_str(iter->sysfs_bus_id);
        device->sysfs_id    = safe_str(iter->sysfs_id);
        devices.emplace_back(std::move(device));
    }

    hd_free_hd_list(hd_list_obj);
    hd_free_hd_data(hd_data.get());
}
}  // namespace

auto Data::initialize_data() noexcept -> Data {
    Data res;

    fillDevices(hw_pci, res.PCIDevices);
    fillDevices(hw_usb, res.USBDevices);

    res.updateConfigData();

    return res;
}

void Data::updateInstalledConfigData() noexcept {
    // Clear config vectors in each device element
    for (const auto& PCIDevice : PCIDevices) {
        PCIDevice->installed_configs.clear();
    }

    for (const auto& USBDevice : USBDevices) {
        USBDevice->installed_configs.clear();
    }

    installedPCIConfigs.clear();
    installedUSBConfigs.clear();

    // Refill data
    fillInstalledConfigs("PCI");
    fillInstalledConfigs("USB");

    setMatchingConfigs(PCIDevices, installedPCIConfigs, true);
    setMatchingConfigs(USBDevices, installedUSBConfigs, true);
}

void Data::fillInstalledConfigs(const std::string_view& type) noexcept {
    const auto& db_path = ("USB" == type) ? consts::MHWD_USB_DATABASE_DIR : consts::MHWD_PCI_DATABASE_DIR;
    auto* configs       = ("USB" == type) ? &installedUSBConfigs : &installedPCIConfigs;

    const auto& config_paths = get_recursive_directory_listing(db_path, consts::MHWD_CONFIG_NAME);
    for (const auto& config_path : config_paths) {
        auto config = std::make_unique<Config>(config_path, type.data());

        if (config->read_file(config_path)) {
            configs->push_back(std::move(config));
        } else {
            invalidConfigs.emplace_back(std::move(config));
        }
    }
}

void Data::getAllDevicesOfConfig(const config_t& config, list_of_devices_t& found_devices) const noexcept {
    const auto& devices = ("USB" == config->type) ? USBDevices : PCIDevices;
    ::mhwd::getAllDevicesOfConfig(devices, config, found_devices);
}

auto Data::getAllDependenciesToInstall(const config_t& config) const noexcept -> list_of_configs_t {
    const auto& installed_configs = ("USB" == config->type) ? installedUSBConfigs : installedPCIConfigs;
    list_of_configs_t depends;
    getAllDependenciesToInstall(config, installed_configs, &depends);
    return depends;
}

void Data::getAllDependenciesToInstall(const config_t& config, const list_of_configs_t& installed_configs, list_of_configs_t* dependencies) const noexcept {
    for (const auto& configDependency : config->dependencies) {
        auto found = ranges::find_if(installed_configs,
                         [configDependency](const auto& tmp) -> bool {
                             return (tmp->name == configDependency);
                         })
            != installed_configs.end();

        if (!found) {
            found = ranges::find_if(*dependencies,
                        [configDependency](const auto& tmp) -> bool {
                            return (tmp->name == configDependency);
                        })
                != dependencies->end();

            if (!found) {
                // Add to vector and check for further sub depends...
                const auto dependency_db_config{getDatabaseConfig(configDependency, config->type)};
                if (nullptr != dependency_db_config) {
                    dependencies->emplace_back(dependency_db_config);
                    getAllDependenciesToInstall(dependency_db_config, installed_configs, dependencies);
                }
            }
        }
    }
}

auto Data::getDatabaseConfig(const std::string_view& config_name, const std::string_view& config_type) const noexcept -> config_t {
    const auto& allConfigs = ("USB" == config_type) ? allUSBConfigs : allPCIConfigs;
    const auto& result     = ranges::find_if(allConfigs.begin(), allConfigs.end(), [&config_name](const auto& config) { return config->name == config_name; });
    return (result != allConfigs.end()) ? *result : nullptr;
}

auto Data::getAllLocalConflicts(const config_t& config) const noexcept -> list_of_configs_t {
    list_of_configs_t conflicts;
    auto dependencies            = getAllDependenciesToInstall(config);
    const auto& installedConfigs = ("USB" == config->type) ? installedUSBConfigs : installedPCIConfigs;

    // Add self to local dependencies vector
    dependencies.emplace_back(config);

    // Loop thru all MHWD config dependencies (not pacman dependencies)
    for (const auto& dependency : dependencies) {
        // Loop through all MHWD config conflicts
        for (const auto& dependencyConflict : dependency->conflicts) {
            // Then loop through all already installed configs. If there are no configs installed, there can not be a conflict
            for (const auto& installedConfig : installedConfigs) {
                // Skip yourself
                /* clang-format off */
                if (installedConfig->name == config->name) { continue; }
                /* clang-format on */

                // Does one of the installed configs conflict one of the to-be-installed configs?
                if (fnmatch(dependencyConflict.c_str(), installedConfig->name.c_str(), FNM_CASEFOLD) == 0) {
                    // Check if conflicts is already in the conflicts vector
                    const bool found = ranges::find_if(conflicts.begin(), conflicts.end(),
                                           [&dependencyConflict](const auto& tmp) {
                                               return tmp->name == dependencyConflict;
                                           })
                        != conflicts.end();
                    // If not, add it to the conflicts vector. This will now be shown to the user.
                    if (!found) {
                        conflicts.push_back(installedConfig);
                        break;
                    }
                }
            }
        }
    }

    return conflicts;
}

auto Data::getAllLocalRequirements(const config_t& config) const noexcept -> list_of_configs_t {
    list_of_configs_t requirements;
    const auto* installedConfigs = ("USB" == config->type) ? &installedUSBConfigs : &installedPCIConfigs;

    // Check if this config is required by another installed config
    for (const auto& installedConfig : *installedConfigs) {
        for (const auto& dependency : installedConfig->dependencies) {
            if (dependency == config->name) {
                const bool found = ranges::find_if(requirements.begin(), requirements.end(),
                                       [&installedConfig](const config_t& req) {
                                           return req->name == installedConfig->name;
                                       })
                    != requirements.end();

                if (!found) {
                    requirements.emplace_back(installedConfig);
                    break;
                }
            }
        }
    }

    return requirements;
}

void Data::fillAllConfigs(const std::string_view& type) noexcept {
    const auto& conf_path   = ("USB" == type) ? consts::MHWD_USB_CONFIG_DIR : consts::MHWD_PCI_CONFIG_DIR;
    const auto& configPaths = get_recursive_directory_listing(conf_path, consts::MHWD_CONFIG_NAME);
    auto* configs           = ("USB" == type) ? &allUSBConfigs : &allPCIConfigs;

    for (auto&& configPath : configPaths) {
        auto config = std::make_unique<Config>(configPath, type.data());

        if (config->read_file(configPath)) {
            configs->emplace_back(std::move(config));
        } else {
            invalidConfigs.emplace_back(std::move(config));
        }
    }
}

void Data::updateConfigData() noexcept {
    for (const auto& PCIDevice : PCIDevices) {
        PCIDevice->available_configs.clear();
    }

    for (const auto& USBDevice : USBDevices) {
        USBDevice->available_configs.clear();
    }

    allPCIConfigs.clear();
    allUSBConfigs.clear();

    fillAllConfigs("PCI");
    fillAllConfigs("USB");

    setMatchingConfigs(PCIDevices, allPCIConfigs, false);
    setMatchingConfigs(USBDevices, allUSBConfigs, false);

    updateInstalledConfigData();
}

}  // namespace mhwd
