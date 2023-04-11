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

#include <algorithm>
#include <filesystem>  // for recursive_directory_iterator
#include <iomanip>     // for setw, setfill
#include <string>      // for string
#include <vector>      // for vector

#include <fmt/core.h>

#if defined(__clang__)
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wold-style-cast"
#pragma clang diagnostic ignored "-Wimplicit-int-conversion"

#include <range/v3/algorithm/find_if.hpp>
#include <range/v3/algorithm/transform.hpp>

#pragma clang diagnostic pop
#else
#include <ranges>
namespace ranges = std::ranges;
#endif

namespace fs = std::filesystem;
using namespace std::string_view_literals;

namespace mhwd {

namespace {
void getAllDevicesOfConfig(const list_of_devices_t& devices, const profile_t& config, list_of_devices_t& found_devices) noexcept {
    found_devices.clear();

    for (auto&& hwdID : config->hwd_ids) {
        bool found_device{};
        // Check all devices
        for (auto&& i_device : devices) {
            // Check class ids
            bool found = ranges::find_if(hwdID.class_ids, [i_device](const auto& classID) {
                const auto& classID_conv = std::string(classID);
                return fnmatch(classID_conv.c_str(), i_device->class_id.c_str(), FNM_CASEFOLD) == 0;
            }) != hwdID.class_ids.end();

            if (found) {
                // Check blacklisted class ids
                found = ranges::find_if(hwdID.blacklisted_class_ids, [i_device](const auto& blacklistedClassID) {
                    const auto& blacklistedClassID_conv = std::string(blacklistedClassID);
                    return fnmatch(blacklistedClassID_conv.c_str(), i_device->class_id.c_str(), FNM_CASEFOLD) == 0;
                }) != hwdID.blacklisted_class_ids.end();

                if (!found) {
                    // Check vendor ids
                    found = ranges::find_if(hwdID.vendor_ids, [i_device](const auto& vendorID) {
                        const auto& vendorID_conv = std::string(vendorID);
                        return fnmatch(vendorID_conv.c_str(), i_device->vendor_id.c_str(), FNM_CASEFOLD) == 0;
                    }) != hwdID.vendor_ids.end();

                    if (found) {
                        // Check blacklisted vendor ids
                        found = ranges::find_if(hwdID.blacklisted_vendor_ids, [i_device](const auto& blacklistedVendorID) {
                            const auto& blacklistedVendorID_conv = std::string(blacklistedVendorID);
                            return fnmatch(blacklistedVendorID_conv.c_str(), i_device->vendor_id.c_str(), FNM_CASEFOLD) == 0;
                        }) != hwdID.blacklisted_vendor_ids.end();

                        if (!found) {
                            // Check device ids
                            found = ranges::find_if(hwdID.device_ids, [i_device](const auto& deviceID) {
                                const auto& deviceID_conv = std::string(deviceID);
                                return fnmatch(deviceID_conv.c_str(), i_device->device_id.c_str(), FNM_CASEFOLD) == 0;
                            }) != hwdID.device_ids.end();

                            if (found) {
                                // Check blacklisted device ids
                                found = ranges::find_if(hwdID.blacklisted_device_ids, [i_device](const auto& blacklistedDeviceID) {
                                    const auto& blacklistedDeviceID_conv = std::string(blacklistedDeviceID);
                                    return fnmatch(blacklistedDeviceID_conv.c_str(), i_device->device_id.c_str(), FNM_CASEFOLD) == 0;
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

void addConfigSorted(list_of_configs_t& configs, const profile_t& newConfig) noexcept {
    const bool found = ranges::find_if(configs.begin(), configs.end(),
                           [&newConfig](const profile_t& config) {
                               return std::string(newConfig->name) == std::string(config->name);
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

void setMatchingConfig(const profile_t& config,
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
        device->type        = (item == hw_usb) ? "USB"sv : "PCI"sv;
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
    fillInstalledConfigs("PCI"sv);
    fillInstalledConfigs("USB"sv);

    setMatchingConfigs(PCIDevices, installedPCIConfigs, true);
    setMatchingConfigs(USBDevices, installedUSBConfigs, true);
}

void Data::fillInstalledConfigs(std::string_view type) noexcept {
    const auto& db_path = ("USB"sv == type) ? consts::MHWD_USB_DATABASE_DIR : consts::MHWD_PCI_DATABASE_DIR;
    auto* configs       = ("USB"sv == type) ? &installedUSBConfigs : &installedPCIConfigs;

    for (const fs::path& dir_entry : fs::directory_iterator{db_path}) {
        /* clang-format off */
        if (fs::is_directory(dir_entry) || dir_entry.filename() != consts::CHWD_CONFIG_FILE) { continue; }
        /* clang-format on */

        auto profiles = chwd::parse_profiles(dir_entry.c_str(), type);
        if (profiles) {
            ranges::transform(
                profiles.value(), std::back_inserter(*configs),
                [](auto&& profile) {
                    return std::make_unique<chwd::Profile>(std::forward<decltype(profile)>(profile));
                });
        }

        auto invalid_profiles = chwd::get_invalid_profiles(dir_entry.c_str()).value_or(chwd::vec_str_t{});
        ranges::transform(
            invalid_profiles, std::back_inserter(invalidConfigs),
            [](auto&& profile) {
                return std::string{profile.c_str()};
            });
    }
    std::sort(configs->begin(), configs->end(), [](auto lhs, auto rhs) { return lhs->priority > rhs->priority; });
}

void Data::getAllDevicesOfConfig(const profile_t& config, list_of_devices_t& found_devices) const noexcept {
    const auto& devices = ("USB"sv == std::string(config->prof_type)) ? USBDevices : PCIDevices;
    ::mhwd::getAllDevicesOfConfig(devices, config, found_devices);
}

auto Data::getDatabaseConfig(const std::string_view& config_name, const std::string_view& config_type) const noexcept -> profile_t {
    const auto& allConfigs = ("USB"sv == config_type) ? allUSBConfigs : allPCIConfigs;
    const auto& result     = ranges::find_if(allConfigs.begin(), allConfigs.end(), [&config_name](const auto& config) { return std::string(config->name) == config_name; });
    return (result != allConfigs.end()) ? *result : nullptr;
}

void Data::fillAllConfigs(std::string_view type) noexcept {
    const auto& conf_path = ("USB"sv == type) ? consts::CHWD_USB_CONFIG_DIR : consts::CHWD_PCI_CONFIG_DIR;
    auto* configs         = ("USB"sv == type) ? &allUSBConfigs : &allPCIConfigs;

    for (const fs::path& dir_entry : fs::directory_iterator{conf_path}) {
        const auto& config_file_path = fmt::format("{}/{}", dir_entry.c_str(), consts::CHWD_CONFIG_FILE);
        /* clang-format off */
        if (!fs::exists(config_file_path)) { continue; }
        /* clang-format on */

        auto profiles = chwd::parse_profiles(config_file_path, type);
        if (profiles) {
            ranges::transform(
                profiles.value(), std::back_inserter(*configs),
                [](auto&& profile) {
                    return std::make_unique<chwd::Profile>(std::forward<decltype(profile)>(profile));
                });
        }

        auto invalid_profiles = chwd::get_invalid_profiles(config_file_path).value_or(chwd::vec_str_t{});
        ranges::transform(
            invalid_profiles, std::back_inserter(invalidConfigs),
            [](auto&& profile) {
                return std::string{profile.c_str()};
            });
    }

    std::sort(configs->begin(), configs->end(), [](auto lhs, auto rhs) { return lhs->priority > rhs->priority; });
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

    fillAllConfigs("PCI"sv);
    fillAllConfigs("USB"sv);

    setMatchingConfigs(PCIDevices, allPCIConfigs, false);
    setMatchingConfigs(USBDevices, allUSBConfigs, false);

    updateInstalledConfigData();
}

}  // namespace mhwd
