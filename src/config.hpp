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

#ifndef CONFIG_HPP
#define CONFIG_HPP

#include "enums.hpp"
#include "vita/string.hpp"

#include <string>
#include <utility>
#include <vector>

namespace mhwd {

struct HardwareID {
    std::vector<std::string> class_ids;
    std::vector<std::string> vendor_ids;
    std::vector<std::string> device_ids;
    std::vector<std::string> blacklisted_class_ids;
    std::vector<std::string> blacklisted_vendor_ids;
    std::vector<std::string> blacklisted_device_ids;
};

struct Config final {
    Config(const std::string_view& configPath, std::string conf_type)
      : type(std::move(conf_type)), base_path(configPath.substr(0, configPath.find_last_of('/'))),
        config_path(configPath) { }

    bool read_file(const std::string_view& file_path) noexcept;

    bool is_freedriver{true};
    std::int32_t priority{};

    std::string type;
    std::string base_path;
    std::string config_path;
    std::string name;
    std::string info;
    std::string version;

    std::vector<std::string> conflicts;
    std::vector<std::string> dependencies;

    std::vector<HardwareID> hwd_ids{1};
};

}  // namespace mhwd

#endif  // CONFIG_HPP
