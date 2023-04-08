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

#ifndef DATA_HPP
#define DATA_HPP

#include "config.hpp"
#include "const.hpp"
#include "device.hpp"

#if defined(__clang__)
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wnested-anon-types"
#endif

#include <hd.h>

#if defined(__clang__)
#pragma clang diagnostic pop
#endif
#include <sys/stat.h>
#include <sys/types.h>

#include <memory>
#include <string>
#include <vector>

#include "vita/string.hpp"

namespace mhwd {

using profile_t         = std::shared_ptr<Profile>;
using device_t          = std::shared_ptr<Device>;
using list_of_configs_t = std::vector<profile_t>;
using list_of_devices_t = std::vector<device_t>;

class Data final {
 public:
    Data()  = default;
    ~Data() = default;

    struct Environment {
        bool syncPackageManagerDatabase = true;
        std::string PMCachePath{consts::MHWD_PM_CACHE_DIR};
        std::string PMConfigPath{consts::MHWD_PM_CONFIG};
        std::string PMRootPath{consts::MHWD_PM_ROOT};
    };

    static auto initialize_data() noexcept -> Data;

    Environment environment;
    list_of_devices_t USBDevices;
    list_of_devices_t PCIDevices;
    list_of_configs_t installedUSBConfigs;
    list_of_configs_t installedPCIConfigs;
    list_of_configs_t allUSBConfigs;
    list_of_configs_t allPCIConfigs;
    std::vector<std::string> invalidConfigs;

    void updateInstalledConfigData() noexcept;
    void getAllDevicesOfConfig(const profile_t& config, list_of_devices_t& found_devices) const noexcept;

    [[nodiscard]] auto getDatabaseConfig(const std::string_view& config_name, const std::string_view& config_type) const noexcept -> profile_t;

 private:
    void fillInstalledConfigs(std::string_view type) noexcept;
    void fillAllConfigs(std::string_view type) noexcept;
    void updateConfigData() noexcept;
};

}  // namespace mhwd

#endif  // DATA_HPP
