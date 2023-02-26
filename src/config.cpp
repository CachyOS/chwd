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

#include "config.hpp"
#include "utils.hpp"

#include <filesystem>  // for path
#include <fstream>     // for ifstream
#include <string>      // for string
#include <vector>      // for vector

#include <fmt/compile.h>
#include <fmt/core.h>

namespace fs = std::filesystem;

namespace mhwd {

namespace {
auto split_value(const Vita::string& str, const Vita::string& onlyEnding = Vita::string{""}) noexcept -> std::vector<std::string> {
    static constexpr auto SPLIT_OFFSET = 5;

    std::vector<std::string> result{};
    auto work = str.to_lower().explode(" ");
    for (auto&& iter : work) {
        if ((!iter.empty()) && onlyEnding.empty()) {
            result.push_back(iter);
        } else if ((!iter.empty()) && (Vita::string{iter}.explode(".").back() == onlyEnding)
            && (iter.size() > SPLIT_OFFSET)) {
            result.push_back(std::string{iter.substr(0, iter.size() - SPLIT_OFFSET)});
        }
    }

    return result;
}

auto get_proper_config_path(const Vita::string& str, const std::string_view& base_config_path) noexcept -> Vita::string {
    const auto& temp = str.trim();
    if ((temp.empty()) || ("/" == temp.substr(0, 1))) {
        return temp;
    }

    fs::path final_config_path{base_config_path};
    final_config_path /= temp.data();
    return Vita::string{final_config_path.c_str()};
}
}  // namespace

bool Config::read_file(const std::string_view& file_path) noexcept {
    std::ifstream file(file_path.data());
    /* clang-format off */
    if (!file) { return false; }
    /* clang-format on */

    while (!file.eof()) {
        Vita::string line{};
        std::getline(file, line);

        auto pos = line.find_first_of('#');
        if (pos != std::string::npos) {
            line.erase(pos);
        }
        /* clang-format off */
        if (line.trim().empty()) { continue; }
        /* clang-format on */

        const auto parts = line.explode("=");
        const auto key   = parts.front().trim().to_lower();
        auto value       = parts.back().trim("\"").trim();

        // Read in extern file
        if ((value.size() > 1) && (">" == value.substr(0, 1))) {
            std::ifstream extern_file(get_proper_config_path(Vita::string{value.substr(1)}, base_path).c_str());
            /* clang-format off */
            if (!extern_file.is_open()) { return false; }
            /* clang-format on */

            value.clear();
            while (!extern_file.eof()) {
                std::getline(extern_file, line);

                pos = line.find_first_of('#');
                /* clang-format off */
                if (std::string::npos != pos) { line.erase(pos); }
                if (line.trim().empty()) { continue; }
                /* clang-format on */

                value += fmt::format(FMT_COMPILE(" {}"), line.trim());
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
            name = value.to_lower();
            break;
        case mhwd::utils::hash_compile_time("version"):
            version = value;
            break;
        case mhwd::utils::hash_compile_time("info"):
            info = value;
            break;
        case mhwd::utils::hash_compile_time("priority"):
            priority = value.convert<std::int32_t>();
            break;
        case mhwd::utils::hash_compile_time("freedriver"):
            value         = value.to_lower();
            is_freedriver = !(value == "false");
            break;
        case mhwd::utils::hash_compile_time("classids"):
            // Add new HardwareIDs group to vector if vector is not empty
            if (!hwd_ids.back().class_ids.empty()) {
                hwd_ids.emplace_back();
            }

            hwd_ids.back().class_ids = split_value(value);
            break;
        case mhwd::utils::hash_compile_time("vendorids"):
            // Add new HardwareIDs group to vector if vector is not empty
            if (!hwd_ids.back().vendor_ids.empty()) {
                hwd_ids.emplace_back();
            }

            hwd_ids.back().vendor_ids = split_value(value);
            break;
        case mhwd::utils::hash_compile_time("deviceids"):
            // Add new HardwareIDs group to vector if vector is not empty
            if (!hwd_ids.back().device_ids.empty()) {
                hwd_ids.emplace_back();
            }

            hwd_ids.back().device_ids = split_value(value);
            break;
        case mhwd::utils::hash_compile_time("blacklistedclassids"):
            hwd_ids.back().blacklisted_class_ids = split_value(value);
            break;
        case mhwd::utils::hash_compile_time("blacklistedvendorids"):
            hwd_ids.back().blacklisted_vendor_ids = split_value(value);
            break;
        case mhwd::utils::hash_compile_time("blacklisteddeviceids"):
            hwd_ids.back().blacklisted_device_ids = split_value(value);
            break;
        case mhwd::utils::hash_compile_time("mhwddepends"):
            dependencies = split_value(value);
            break;
        case mhwd::utils::hash_compile_time("mhwdconflicts"):
            conflicts = split_value(value);
            break;
        }
    }

    // Append * to all empty vectors
    for (auto& hwd_id : hwd_ids) {
        if (hwd_id.class_ids.empty()) {
            hwd_id.class_ids.emplace_back("*");
        }

        if (hwd_id.vendor_ids.empty()) {
            hwd_id.vendor_ids.emplace_back("*");
        }

        if (hwd_id.device_ids.empty()) {
            hwd_id.device_ids.emplace_back("*");
        }
    }

    return !name.empty();
}

}  // namespace mhwd
