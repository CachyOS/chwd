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

class Data final {
 public:
    Data();
    ~Data() = default;

    struct Environment {
        std::string PMCachePath{consts::MHWD_PM_CACHE_DIR};
        std::string PMConfigPath{consts::MHWD_PM_CONFIG};
        std::string PMRootPath{consts::MHWD_PM_ROOT};
        bool syncPackageManagerDatabase = true;
    };

    Environment environment;
    std::vector<std::shared_ptr<Device>> USBDevices;
    std::vector<std::shared_ptr<Device>> PCIDevices;
    std::vector<std::shared_ptr<Config>> installedUSBConfigs;
    std::vector<std::shared_ptr<Config>> installedPCIConfigs;
    std::vector<std::shared_ptr<Config>> allUSBConfigs;
    std::vector<std::shared_ptr<Config>> allPCIConfigs;
    std::vector<std::shared_ptr<Config>> invalidConfigs;

    void updateInstalledConfigData();
    void getAllDevicesOfConfig(std::shared_ptr<Config> config, std::vector<std::shared_ptr<Device>>& foundDevices);

    std::vector<std::shared_ptr<Config>> getAllDependenciesToInstall(std::shared_ptr<Config> config);
    void getAllDependenciesToInstall(std::shared_ptr<Config> config,
        std::vector<std::shared_ptr<Config>>& installedConfigs,
        std::vector<std::shared_ptr<Config>>* depends);
    std::shared_ptr<Config> getDatabaseConfig(const std::string configName,
        const std::string configType);
    std::vector<std::shared_ptr<Config>> getAllLocalConflicts(std::shared_ptr<Config> config);
    std::vector<std::shared_ptr<Config>> getAllLocalRequirements(std::shared_ptr<Config> config);

 private:
    void getAllDevicesOfConfig(const std::vector<std::shared_ptr<Device>>& devices,
        std::shared_ptr<Config> config, std::vector<std::shared_ptr<Device>>& foundDevices);
    void fillInstalledConfigs(std::string type);
    void fillDevices(hw_item hw, std::vector<std::shared_ptr<Device>>& devices);
    void fillAllConfigs(std::string type);
    void setMatchingConfigs(const std::vector<std::shared_ptr<Device>>& devices,
        std::vector<std::shared_ptr<Config>>& configs, bool setAsInstalled);
    void setMatchingConfig(std::shared_ptr<Config> config, const std::vector<std::shared_ptr<Device>>& devices,
        bool setAsInstalled);
    void addConfigSorted(std::vector<std::shared_ptr<Config>>& configs, std::shared_ptr<Config> newConfig);
    std::vector<std::string> getRecursiveDirectoryFileList(const std::string_view& directoryPath, const std::string_view& onlyFilename = "");

    Vita::string get_proper_config_path(const Vita::string& str, const std::string_view& baseConfigPath);
    void updateConfigData();

    Vita::string from_hex(uint16_t hexnum, int fill);
    std::string from_CharArray(char* c);
};

}  // namespace mhwd

#endif  // DATA_HPP
