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

#include "config.hpp"
#include "utils.hpp"

#include <filesystem>
#include <fstream>
#include <string>
#include <vector>

#include <fmt/core.h>

namespace fs = std::filesystem;

namespace mhwd {

namespace {
std::vector<std::string> splitValue(const Vita::string& str, const Vita::string& onlyEnding = "") {
    auto work = str.toLower().explode(" ");
    std::vector<std::string> result;

    for (auto&& iter : work) {
        if ((!iter.empty()) && onlyEnding.empty()) {
            result.push_back(iter);
        } else if ((!iter.empty()) && (Vita::string{iter}.explode(".").back() == onlyEnding)
            && (iter.size() > 5)) {
            result.push_back(std::string{iter}.substr(0, iter.size() - 5));
        }
    }

    return result;
}

Vita::string get_proper_config_path(const Vita::string& str, const std::string_view& base_config_path) {
    const auto& temp = str.trim();
    if ((temp.empty()) || ("/" == temp.substr(0, 1))) {
        return temp;
    }

    fs::path p{base_config_path};
    p /= temp.data();

    return p.c_str();
}
}  // namespace

bool Config::read_file(const std::string_view& configPath) {
    std::ifstream file(configPath.data());

    if (!file) {
        return false;
    }

    Vita::string line;
    Vita::string key;
    Vita::string value;
    std::vector<Vita::string> parts;

    while (!file.eof()) {
        std::getline(file, line);

        auto pos = line.find_first_of('#');
        if (pos != std::string::npos) {
            line.erase(pos);
        }

        if (line.trim().empty()) {
            continue;
        }

        parts = line.explode("=");
        key   = parts.front().trim().toLower();
        value = parts.back().trim("\"").trim();

        // Read in extern file
        if ((value.size() > 1) && (">" == value.substr(0, 1))) {
            std::ifstream extern_file(get_proper_config_path(value.substr(1), base_path).c_str());
            if (!extern_file.is_open()) {
                return false;
            }

            value.clear();

            while (!extern_file.eof()) {
                std::getline(extern_file, line);

                pos = line.find_first_of('#');
                if (std::string::npos != pos) {
                    line.erase(pos);
                }

                if (line.trim().empty()) {
                    continue;
                }

                value += fmt::format(" {}", line.trim());
            }

            value = value.trim();

            // remove all multiple spaces
            while (value.find("  ") != std::string::npos) {
                value = value.replace("  ", " ");
            }
        }

        switch (mhwd::utils::hash(key.c_str())) {
        case mhwd::utils::hash_compile_time("include"):
            read_file(get_proper_config_path(value, base_path));
            break;
        case mhwd::utils::hash_compile_time("name"):
            name = value.toLower();
            break;
        case mhwd::utils::hash_compile_time("version"):
            version = value;
            break;
        case mhwd::utils::hash_compile_time("info"):
            info = value;
            break;
        case mhwd::utils::hash_compile_time("priority"):
            priority = value.convert<int>();
            break;
        case mhwd::utils::hash_compile_time("freedriver"):
            value         = value.toLower();
            is_freedriver = !(value == "false");
            break;
        case mhwd::utils::hash_compile_time("classids"):
            // Add new HardwareIDs group to vector if vector is not empty
            if (!hwd_ids.back().class_ids.empty()) {
                hwd_ids.emplace_back(HardwareID{});
            }

            hwd_ids.back().class_ids = splitValue(value);
            break;
        case mhwd::utils::hash_compile_time("vendorids"):
            // Add new HardwareIDs group to vector if vector is not empty
            if (!hwd_ids.back().vendor_ids.empty()) {
                hwd_ids.emplace_back(HardwareID{});
            }

            hwd_ids.back().vendor_ids = splitValue(value);
            break;
        case mhwd::utils::hash_compile_time("deviceids"):
            // Add new HardwareIDs group to vector if vector is not empty
            if (!hwd_ids.back().device_ids.empty()) {
                hwd_ids.emplace_back(HardwareID{});
            }

            hwd_ids.back().device_ids = splitValue(value);
            break;
        case mhwd::utils::hash_compile_time("blacklistedclassids"):
            hwd_ids.back().blacklisted_class_ids = splitValue(value);
            break;
        case mhwd::utils::hash_compile_time("blacklistedvendorids"):
            hwd_ids.back().blacklisted_vendor_ids = splitValue(value);
            break;
        case mhwd::utils::hash_compile_time("blacklisteddeviceids"):
            hwd_ids.back().blacklisted_device_ids = splitValue(value);
            break;
        case mhwd::utils::hash_compile_time("mhwddepends"):
            dependencies = splitValue(value);
            break;
        case mhwd::utils::hash_compile_time("mhwdconflicts"):
            conflicts = splitValue(value);
            break;
        }
    }

    // Append * to all empty vectors
    for (auto& hwdid : hwd_ids) {
        if (hwdid.class_ids.empty()) {
            hwdid.class_ids.emplace_back("*");
        }

        if (hwdid.vendor_ids.empty()) {
            hwdid.vendor_ids.emplace_back("*");
        }

        if (hwdid.device_ids.empty()) {
            hwdid.device_ids.emplace_back("*");
        }
    }

    return !name.empty();
}

}  // namespace mhwd
