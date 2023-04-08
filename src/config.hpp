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
#include <optional>

namespace mhwd {

struct HardwareID {
    std::vector<std::string> class_ids;
    std::vector<std::string> vendor_ids;
    std::vector<std::string> device_ids;
    std::vector<std::string> blacklisted_class_ids;
    std::vector<std::string> blacklisted_vendor_ids;
    std::vector<std::string> blacklisted_device_ids;
};

struct Profile {
    bool is_nonfree{false};

    std::string type{};
    std::string name{};
    std::string desc{};
    std::int32_t priority{};

    std::vector<HardwareID> hwd_ids{1};

    static auto parse_profiles(const std::string_view& file_path, std::string_view type_name) noexcept -> std::optional<std::vector<Profile>>;
    static auto get_invalid_profiles(const std::string_view& file_path) noexcept -> std::optional<std::vector<std::string>>;
    static auto write_profile_to_file(const std::string_view& file_path, const Profile& profile) noexcept -> bool;
};

}  // namespace mhwd

#endif  // CONFIG_HPP
