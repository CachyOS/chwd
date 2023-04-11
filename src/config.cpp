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

#include <fmt/core.h>
#include <fmt/compile.h>

namespace chwd {

using parse_profile_t = decltype(chwd::parse_profiles_ffi("", ""));
using vec_str_t = decltype(chwd::get_invalid_profiles_ffi(""));

auto parse_profiles(std::string_view file_path, std::string_view type) noexcept -> std::optional<parse_profile_t> {
    try {
        return chwd::parse_profiles_ffi(file_path.data(), type.data());
    } catch(const std::exception &e) {
        fmt::print(stderr, FMT_COMPILE("Failed to parse: '{}'\n"), e.what());
        return std::nullopt;
    }
}
auto get_invalid_profiles(std::string_view file_path) noexcept -> std::optional<vec_str_t> {
    try {
        return chwd::get_invalid_profiles_ffi(file_path.data());
    } catch(const std::exception &e) {
        fmt::print(stderr, FMT_COMPILE("Failed to parse: '{}'\n"), e.what());
        return std::nullopt;
    }
}

}  // namespace chwd
