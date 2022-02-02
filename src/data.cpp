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

#include "data.hpp"

#include <dirent.h>
#include <fnmatch.h>

#include <algorithm>
#include <filesystem>
#include <fstream>
#include <iomanip>
#include <sstream>
#include <string>
#include <vector>

#include <fmt/core.h>

namespace fs = std::filesystem;

namespace mhwd {

Data::Data() {
    fillDevices(hw_pci, PCIDevices);
    fillDevices(hw_usb, USBDevices);

    updateConfigData();
}

void Data::updateInstalledConfigData() {
    // Clear config vectors in each device element

    for (auto& PCIDevice : PCIDevices) {
        PCIDevice->installed_configs.clear();
    }

    for (auto& USBDevice : USBDevices) {
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

void Data::fillInstalledConfigs(std::string type) {
    std::vector<std::string> configPaths;
    std::vector<std::shared_ptr<Config>>* configs;

    if ("USB" == type) {
        configs     = &installedUSBConfigs;
        configPaths = getRecursiveDirectoryFileList(consts::MHWD_USB_DATABASE_DIR, consts::MHWD_CONFIG_NAME);
    } else {
        configs     = &installedPCIConfigs;
        configPaths = getRecursiveDirectoryFileList(consts::MHWD_PCI_DATABASE_DIR, consts::MHWD_CONFIG_NAME);
    }

    for (const auto& configPath : configPaths) {
        Config* config = new Config(configPath, type);

        if (config->read_file(configPath)) {
            configs->push_back(std::shared_ptr<Config>{config});
        } else {
            invalidConfigs.push_back(std::shared_ptr<Config>{config});
        }
    }
}

void Data::getAllDevicesOfConfig(std::shared_ptr<Config> config, std::vector<std::shared_ptr<Device>>& foundDevices) {
    const auto& devices = ("USB" == config->type) ? USBDevices : PCIDevices;
    getAllDevicesOfConfig(devices, config, foundDevices);
}

void Data::getAllDevicesOfConfig(const std::vector<std::shared_ptr<Device>>& devices,
    std::shared_ptr<Config> config,
    std::vector<std::shared_ptr<Device>>& foundDevices) {
    foundDevices.clear();

    for (auto&& hwdID = config->hwd_ids.begin();
         hwdID != config->hwd_ids.end(); ++hwdID) {
        bool foundDevice = false;
        // Check all devices
        for (auto&& i_device = devices.begin(); i_device != devices.end();
             ++i_device) {
            // Check class ids
            bool found = std::find_if(hwdID->class_ids.begin(), hwdID->class_ids.end(), [i_device](const std::string& classID) {
                return !fnmatch(classID.c_str(), (*i_device)->class_id.c_str(), FNM_CASEFOLD);
            }) != hwdID->class_ids.end();

            if (found) {
                // Check blacklisted class ids
                found = std::find_if(hwdID->blacklisted_class_ids.begin(), hwdID->blacklisted_class_ids.end(), [i_device](const std::string& blacklistedClassID) {
                    return !fnmatch(blacklistedClassID.c_str(), (*i_device)->class_id.c_str(), FNM_CASEFOLD);
                }) != hwdID->blacklisted_class_ids.end();

                if (!found) {
                    // Check vendor ids
                    found = std::find_if(hwdID->vendor_ids.begin(), hwdID->vendor_ids.end(), [i_device](const std::string& vendorID) {
                        return !fnmatch(vendorID.c_str(), (*i_device)->vendor_id.c_str(), FNM_CASEFOLD);
                    }) != hwdID->vendor_ids.end();

                    if (found) {
                        // Check blacklisted vendor ids
                        found = std::find_if(hwdID->blacklisted_vendor_ids.begin(), hwdID->blacklisted_vendor_ids.end(), [i_device](const std::string& blacklistedVendorID) {
                            return !fnmatch(blacklistedVendorID.c_str(), (*i_device)->vendor_id.c_str(), FNM_CASEFOLD);
                        }) != hwdID->blacklisted_vendor_ids.end();

                        if (!found) {
                            // Check device ids
                            found = std::find_if(hwdID->device_ids.begin(), hwdID->device_ids.end(), [i_device](const std::string& deviceID) {
                                return !fnmatch(deviceID.c_str(), (*i_device)->device_id.c_str(), FNM_CASEFOLD);
                            }) != hwdID->device_ids.end();

                            if (found) {
                                // Check blacklisted device ids
                                found = std::find_if(hwdID->blacklisted_device_ids.begin(), hwdID->blacklisted_device_ids.end(), [i_device](const std::string& blacklistedDeviceID) {
                                    return !fnmatch(blacklistedDeviceID.c_str(), (*i_device)->device_id.c_str(), FNM_CASEFOLD);
                                }) != hwdID->blacklisted_device_ids.end();
                                if (!found) {
                                    foundDevice = true;
                                    foundDevices.push_back(*i_device);
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

std::vector<std::shared_ptr<Config>> Data::getAllDependenciesToInstall(
    std::shared_ptr<Config> config) {
    std::vector<std::shared_ptr<Config>> depends;
    std::vector<std::shared_ptr<Config>> installedConfigs;

    if ("USB" == config->type) {
        installedConfigs = installedUSBConfigs;
    } else {
        installedConfigs = installedPCIConfigs;
    }

    getAllDependenciesToInstall(config, installedConfigs, &depends);

    return depends;
}

void Data::getAllDependenciesToInstall(std::shared_ptr<Config> config,
    std::vector<std::shared_ptr<Config>>& installedConfigs,
    std::vector<std::shared_ptr<Config>>* dependencies) {
    for (const auto& configDependency : config->dependencies) {
        auto found = std::find_if(installedConfigs.begin(), installedConfigs.end(),
                         [configDependency](const auto& tmp) -> bool {
                             return (tmp->name == configDependency);
                         })
            != installedConfigs.end();

        if (!found) {
            found = std::find_if(dependencies->begin(), dependencies->end(),
                        [configDependency](const auto& tmp) -> bool {
                            return (tmp->name == configDependency);
                        })
                != dependencies->end();

            if (!found) {
                // Add to vector and check for further subdepends...
                auto dependconfig{getDatabaseConfig(configDependency, config->type)};
                if (nullptr != dependconfig) {
                    dependencies->emplace_back(dependconfig);
                    getAllDependenciesToInstall(dependconfig, installedConfigs, dependencies);
                }
            }
        }
    }
}

std::shared_ptr<Config> Data::getDatabaseConfig(const std::string configName,
    const std::string configType) {
    auto allConfigs = ("USB" == configType) ? allUSBConfigs : allPCIConfigs;

    for (auto& config : allConfigs) {
        if (configName == config->name) {
            return config;
        }
    }

    return nullptr;
}

std::vector<std::shared_ptr<Config>> Data::getAllLocalConflicts(std::shared_ptr<Config> config) {
    std::vector<std::shared_ptr<Config>> conflicts;
    auto dependencies     = getAllDependenciesToInstall(config);
    auto installedConfigs = ("USB" == config->type) ? installedUSBConfigs : installedPCIConfigs;

    // Add self to local dependencies vector
    dependencies.emplace_back(config);

    // Loop thru all MHWD config dependencies (not pacman dependencies)
    for (const auto& dependency : dependencies) {
        // Loop thru all MHWD config conflicts
        for (const auto& dependencyConflict : dependency->conflicts) {
            // Then loop thru all already installed configs. If there are no configs installed, there can not be a conflict
            for (auto& installedConfig : installedConfigs) {
                // Skip yourself
                if (installedConfig->name == config->name)
                    continue;
                // Does one of the installed configs conflict one of the to-be-installed configs?
                if (!fnmatch(dependencyConflict.c_str(), installedConfig->name.c_str(), FNM_CASEFOLD)) {
                    // Check if conflicts is already in the conflicts vector
                    const bool found = std::find_if(conflicts.begin(), conflicts.end(),
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

std::vector<std::shared_ptr<Config>> Data::getAllLocalRequirements(std::shared_ptr<Config> config) {
    std::vector<std::shared_ptr<Config>> requirements;
    auto installedConfigs = ("USB" == config->type) ? installedUSBConfigs : installedPCIConfigs;

    // Check if this config is required by another installed config
    for (auto& installedConfig : installedConfigs) {
        for (const auto& dependency : installedConfig->dependencies) {
            if (dependency == config->name) {
                const bool found = std::find_if(requirements.begin(), requirements.end(),
                                       [&installedConfig](const std::shared_ptr<Config>& req) {
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

void Data::fillDevices(hw_item hw, std::vector<std::shared_ptr<Device>>& devices) {
    // Get the hardware devices
    std::unique_ptr<hd_data_t> hd_data{new hd_data_t()};
    hd_t* hd = hd_list(hd_data.get(), hw, 1, nullptr);

    std::unique_ptr<Device> device;
    for (hd_t* hdIter = hd; hdIter; hdIter = hdIter->next) {
        device.reset(new Device());
        device->type        = (hw == hw_usb) ? "USB" : "PCI";
        device->class_id    = from_hex(static_cast<uint16_t>(hdIter->base_class.id), 2) + from_hex(static_cast<uint16_t>(hdIter->sub_class.id), 2).toLower();
        device->vendor_id   = from_hex(static_cast<uint16_t>(hdIter->vendor.id), 4).toLower();
        device->device_id   = from_hex(static_cast<uint16_t>(hdIter->device.id), 4).toLower();
        device->class_name  = from_CharArray(hdIter->base_class.name);
        device->vendor_name = from_CharArray(hdIter->vendor.name);
        device->device_name = from_CharArray(hdIter->device.name);
        device->sysfs_busid = from_CharArray(hdIter->sysfs_bus_id);
        device->sysfs_id    = from_CharArray(hdIter->sysfs_id);
        devices.emplace_back(device.release());
    }

    hd_free_hd_list(hd);
    hd_free_hd_data(hd_data.get());
}

void Data::fillAllConfigs(std::string type) {
    std::vector<std::string> configPaths;
    std::vector<std::shared_ptr<Config>>* configs;

    if ("USB" == type) {
        configs     = &allUSBConfigs;
        configPaths = getRecursiveDirectoryFileList(consts::MHWD_USB_CONFIG_DIR, consts::MHWD_CONFIG_NAME);
    } else {
        configs     = &allPCIConfigs;
        configPaths = getRecursiveDirectoryFileList(consts::MHWD_PCI_CONFIG_DIR, consts::MHWD_CONFIG_NAME);
    }

    for (auto&& configPath : configPaths) {
        std::unique_ptr<Config> config{new Config(configPath, type)};

        if (config->read_file(configPath)) {
            configs->emplace_back(config.release());
        } else {
            invalidConfigs.emplace_back(config.release());
        }
    }
}

std::vector<std::string> Data::getRecursiveDirectoryFileList(const std::string_view& directoryPath, const std::string_view& onlyFilename) {
    std::vector<std::string> list;
    struct dirent* dir = nullptr;
    DIR* d             = opendir(directoryPath.data());
    if (d) {
        while (nullptr != (dir = readdir(d))) {
            const std::string filename{dir->d_name};
            if (("." != filename) && (".." != filename) && ("" != filename)) {
                const auto& filepath{fmt::format("{}/{}", directoryPath, filename)};
                struct stat filestatus;
                lstat(filepath.data(), &filestatus);

                if (S_ISREG(filestatus.st_mode) && (onlyFilename.empty() || (onlyFilename == filename))) {
                    list.push_back(filepath);
                } else if (S_ISDIR(filestatus.st_mode)) {
                    auto templist = getRecursiveDirectoryFileList(filepath.data(), onlyFilename);

                    for (auto&& iter : templist) {
                        list.emplace_back(iter);
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
    if ((temp.size() <= 0) || ("/" == temp.substr(0, 1))) {
        return temp;
    }

    fs::path p{base_config_path};
    p /= temp.data();

    return p.c_str();
}

void Data::updateConfigData() {
    for (auto& PCIDevice : PCIDevices) {
        PCIDevice->available_configs.clear();
    }

    for (auto& USBDevice : USBDevices) {
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

void Data::setMatchingConfigs(const std::vector<std::shared_ptr<Device>>& devices,
    std::vector<std::shared_ptr<Config>>& configs, bool setAsInstalled) {
    for (auto& config : configs) {
        setMatchingConfig(config, devices, setAsInstalled);
    }
}

void Data::setMatchingConfig(std::shared_ptr<Config> config,
    const std::vector<std::shared_ptr<Device>>& devices, bool setAsInstalled) {
    std::vector<std::shared_ptr<Device>> foundDevices;

    getAllDevicesOfConfig(devices, config, foundDevices);

    // Set config to all matching devices
    for (auto& foundDevice : foundDevices) {
        if (setAsInstalled) {
            addConfigSorted(foundDevice->installed_configs, config);
        } else {
            addConfigSorted(foundDevice->available_configs, config);
        }
    }
}

void Data::addConfigSorted(std::vector<std::shared_ptr<Config>>& configs,
    std::shared_ptr<Config> newConfig) {
    const bool found = std::find_if(configs.begin(), configs.end(),
                           [&newConfig](const std::shared_ptr<Config>& config) {
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

Vita::string Data::from_hex(std::uint16_t hexnum, int fill) {
    std::stringstream stream;
    stream << std::hex << std::setfill('0') << std::setw(fill) << hexnum;
    return stream.str();
}

std::string Data::from_CharArray(char* c) {
    if (nullptr == c) {
        return "";
    }

    return std::string(c);
}

}  // namespace mhwd
