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

#include <dirent.h>
#include <fnmatch.h>

#include <algorithm>
#include <filesystem>
#include <iomanip>
#include <string>
#include <vector>

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

#include <fmt/core.h>

namespace fs = std::filesystem;

namespace mhwd {

namespace {
inline Vita::string from_hex(std::uint16_t hexnum, int fill) noexcept {
    std::stringstream stream;
    stream << std::hex << std::setfill('0') << std::setw(fill) << hexnum;
    return stream.str();
}

inline std::string from_CharArray(char* c) noexcept {
    if (nullptr == c) {
        return "";
    }

    return {c};
}

void getAllDevicesOfConfig(const list_of_devices_t& devices, const config_t& config, list_of_devices_t& foundDevices) noexcept {
    foundDevices.clear();

    for (auto&& hwdID : config->hwd_ids) {
        bool foundDevice = false;
        // Check all devices
        for (auto&& i_device : devices) {
            // Check class ids
            bool found = ranges::find_if(hwdID.class_ids, [i_device](const std::string& classID) {
                return !fnmatch(classID.c_str(), i_device->class_id.c_str(), FNM_CASEFOLD);
            }) != hwdID.class_ids.end();

            if (found) {
                // Check blacklisted class ids
                found = ranges::find_if(hwdID.blacklisted_class_ids, [i_device](const std::string& blacklistedClassID) {
                    return !fnmatch(blacklistedClassID.c_str(), i_device->class_id.c_str(), FNM_CASEFOLD);
                }) != hwdID.blacklisted_class_ids.end();

                if (!found) {
                    // Check vendor ids
                    found = ranges::find_if(hwdID.vendor_ids, [i_device](const std::string& vendorID) {
                        return !fnmatch(vendorID.c_str(), i_device->vendor_id.c_str(), FNM_CASEFOLD);
                    }) != hwdID.vendor_ids.end();

                    if (found) {
                        // Check blacklisted vendor ids
                        found = ranges::find_if(hwdID.blacklisted_vendor_ids, [i_device](const std::string& blacklistedVendorID) {
                            return !fnmatch(blacklistedVendorID.c_str(), i_device->vendor_id.c_str(), FNM_CASEFOLD);
                        }) != hwdID.blacklisted_vendor_ids.end();

                        if (!found) {
                            // Check device ids
                            found = ranges::find_if(hwdID.device_ids, [i_device](const std::string& deviceID) {
                                return !fnmatch(deviceID.c_str(), i_device->device_id.c_str(), FNM_CASEFOLD);
                            }) != hwdID.device_ids.end();

                            if (found) {
                                // Check blacklisted device ids
                                found = ranges::find_if(hwdID.blacklisted_device_ids, [i_device](const std::string& blacklistedDeviceID) {
                                    return !fnmatch(blacklistedDeviceID.c_str(), i_device->device_id.c_str(), FNM_CASEFOLD);
                                }) != hwdID.blacklisted_device_ids.end();
                                if (!found) {
                                    foundDevice = true;
                                    foundDevices.push_back(i_device);
                                }
                            }
                        }
                    }
                }
            }
        }

        if (!foundDevice) {
            foundDevices.clear();
            return;
        }
    }
}

void addConfigSorted(list_of_configs_t& configs,
    config_t newConfig) noexcept {
    const bool found = ranges::find_if(configs.begin(), configs.end(),
                           [&newConfig](const config_t& config) {
                               return newConfig->name == config->name;
                           })
        != configs.end();

    if (!found) {
        for (size_t i = 0; i < configs.size(); ++i) {
            auto&& config = configs[i];
            if (newConfig->priority > config->priority) {
                configs.insert(configs.begin() + static_cast<long>(i), newConfig);
                return;
            }
        }
        configs.emplace_back(newConfig);
    }
}

void setMatchingConfig(const config_t& config,
    const list_of_devices_t& devices, bool setAsInstalled) noexcept {
    list_of_devices_t foundDevices;
    ::mhwd::getAllDevicesOfConfig(devices, config, foundDevices);

    // Set config to all matching devices
    for (auto& foundDevice : foundDevices) {
        if (setAsInstalled) {
            addConfigSorted(foundDevice->installed_configs, config);
        } else {
            addConfigSorted(foundDevice->available_configs, config);
        }
    }
}

void setMatchingConfigs(const list_of_devices_t& devices,
    list_of_configs_t& configs, bool setAsInstalled) noexcept {
    for (auto& config : configs) {
        setMatchingConfig(config, devices, setAsInstalled);
    }
}

void fillDevices(hw_item hw, list_of_devices_t& devices) noexcept {
    // Get the hardware devices
    auto hd_data = std::make_unique<hd_data_t>();
    hd_t* hd     = hd_list(hd_data.get(), hw, 1, nullptr);

    for (hd_t* hdIter = hd; hdIter; hdIter = hdIter->next) {
        auto device         = std::make_shared<Device>();
        device->type        = (hw == hw_usb) ? "USB" : "PCI";
        device->class_id    = from_hex(static_cast<uint16_t>(hdIter->base_class.id), 2) + from_hex(static_cast<uint16_t>(hdIter->sub_class.id), 2).toLower();
        device->vendor_id   = from_hex(static_cast<uint16_t>(hdIter->vendor.id), 4).toLower();
        device->device_id   = from_hex(static_cast<uint16_t>(hdIter->device.id), 4).toLower();
        device->class_name  = from_CharArray(hdIter->base_class.name);
        device->vendor_name = from_CharArray(hdIter->vendor.name);
        device->device_name = from_CharArray(hdIter->device.name);
        device->sysfs_busid = from_CharArray(hdIter->sysfs_bus_id);
        device->sysfs_id    = from_CharArray(hdIter->sysfs_id);
        devices.emplace_back(device);
    }

    hd_free_hd_list(hd);
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
    const auto& db_path     = ("USB" == type) ? consts::MHWD_USB_DATABASE_DIR : consts::MHWD_PCI_DATABASE_DIR;
    const auto& configPaths = getRecursiveDirectoryFileList(db_path, consts::MHWD_CONFIG_NAME);
    auto* configs           = ("USB" == type) ? &installedUSBConfigs : &installedPCIConfigs;

    for (const auto& configPath : configPaths) {
        auto config = std::make_shared<Config>(configPath, type.data());

        if (config->read_file(configPath)) {
            configs->push_back(std::move(config));
        } else {
            invalidConfigs.push_back(config);
        }
    }
}

void Data::getAllDevicesOfConfig(const config_t& config, list_of_devices_t& foundDevices) const noexcept {
    const auto& devices = ("USB" == config->type) ? USBDevices : PCIDevices;
    ::mhwd::getAllDevicesOfConfig(devices, config, foundDevices);
}

list_of_configs_t Data::getAllDependenciesToInstall(const config_t& config) noexcept {
    auto installedConfigs = ("USB" == config->type) ? installedUSBConfigs : installedPCIConfigs;
    list_of_configs_t depends;
    getAllDependenciesToInstall(config, installedConfigs, &depends);

    return depends;
}

void Data::getAllDependenciesToInstall(const config_t& config, list_of_configs_t& installedConfigs, list_of_configs_t* dependencies) noexcept {
    for (const auto& configDependency : config->dependencies) {
        auto found = ranges::find_if(installedConfigs,
                         [configDependency](const auto& tmp) -> bool {
                             return (tmp->name == configDependency);
                         })
            != installedConfigs.end();

        if (!found) {
            found = ranges::find_if(*dependencies,
                        [configDependency](const auto& tmp) -> bool {
                            return (tmp->name == configDependency);
                        })
                != dependencies->end();

            if (!found) {
                // Add to vector and check for further subdepends...
                const auto dependconfig{getDatabaseConfig(configDependency, config->type)};
                if (nullptr != dependconfig) {
                    dependencies->emplace_back(dependconfig);
                    getAllDependenciesToInstall(dependconfig, installedConfigs, dependencies);
                }
            }
        }
    }
}

config_t Data::getDatabaseConfig(const std::string_view& configName, const std::string_view& configType) const noexcept {
    const auto allConfigs = ("USB" == configType) ? allUSBConfigs : allPCIConfigs;
    const auto result     = ranges::find_if(allConfigs.begin(), allConfigs.end(), [&configName](const auto& config) { return config->name == configName; });
    return (result != allConfigs.end()) ? *result : nullptr;
}

list_of_configs_t Data::getAllLocalConflicts(const config_t& config) noexcept {
    list_of_configs_t conflicts;
    auto dependencies     = getAllDependenciesToInstall(config);
    auto installedConfigs = ("USB" == config->type) ? installedUSBConfigs : installedPCIConfigs;

    // Add self to local dependencies vector
    dependencies.emplace_back(config);

    // Loop thru all MHWD config dependencies (not pacman dependencies)
    for (const auto& dependency : dependencies) {
        // Loop through all MHWD config conflicts
        for (const auto& dependencyConflict : dependency->conflicts) {
            // Then loop through all already installed configs. If there are no configs installed, there can not be a conflict
            for (auto& installedConfig : installedConfigs) {
                // Skip yourself
                if (installedConfig->name == config->name)
                    continue;
                // Does one of the installed configs conflict one of the to-be-installed configs?
                if (!fnmatch(dependencyConflict.c_str(), installedConfig->name.c_str(), FNM_CASEFOLD)) {
                    // Check if conflicts is already in the conflicts vector
                    const bool found = ranges::find_if(conflicts.begin(), conflicts.end(),
                                           [&dependencyConflict](const auto& tmp) {
                                               return tmp->name == dependencyConflict;
                                           })
                        != conflicts.end();
                    // If not, add it to the conflicts vector. This will now be shown to the user.
                    if (!found) {
                        conflicts.emplace_back(installedConfig);
                        break;
                    }
                }
            }
        }
    }

    return conflicts;
}

list_of_configs_t Data::getAllLocalRequirements(const config_t& config) noexcept {
    list_of_configs_t requirements;
    auto installedConfigs = ("USB" == config->type) ? &installedUSBConfigs : &installedPCIConfigs;

    // Check if this config is required by another installed config
    for (auto& installedConfig : *installedConfigs) {
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
    const auto& configPaths = getRecursiveDirectoryFileList(conf_path, consts::MHWD_CONFIG_NAME);
    auto configs            = ("USB" == type) ? &allUSBConfigs : &allPCIConfigs;

    for (auto&& configPath : configPaths) {
        auto config = std::make_unique<Config>(configPath, type.data());

        if (config->read_file(configPath)) {
            configs->emplace_back(config.release());
        } else {
            invalidConfigs.emplace_back(config.release());
        }
    }
}

std::vector<std::string> Data::getRecursiveDirectoryFileList(const std::string_view& directoryPath, const std::string_view& onlyFilename) const noexcept {
    std::vector<std::string> list;
    struct dirent* dir = nullptr;
    DIR* d             = opendir(directoryPath.data());
    if (d) {
        while (nullptr != (dir = readdir(d))) {
            const std::string_view filename{dir->d_name};
            if (("." != filename) && (".." != filename) && (!filename.empty())) {
                const auto& filepath{fmt::format("{}/{}", directoryPath, filename)};
                struct stat filestatus { };
                lstat(filepath.data(), &filestatus);

                if (S_ISREG(filestatus.st_mode) && (onlyFilename.empty() || (onlyFilename == filename))) {
                    list.push_back(filepath);
                } else if (S_ISDIR(filestatus.st_mode)) {
                    auto&& templist = getRecursiveDirectoryFileList(filepath.data(), onlyFilename);

                    for (auto&& iter : templist) {
                        list.emplace_back(std::move(iter));
                    }
                }
            }
        }

        closedir(d);
    }
    delete dir;
    return list;
}

Vita::string Data::get_proper_config_path(const Vita::string& str, const std::string_view& base_config_path) {
    const auto& temp = str.trim();
    if (temp.empty() || ("/" == temp.substr(0, 1))) {
        return temp;
    }

    fs::path p{base_config_path};
    p /= temp.data();

    return p.c_str();
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
