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

#if defined(__clang__)
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdollar-in-identifier-extension"
#pragma clang diagnostic ignored "-Wsign-conversion"
#elif defined(__GNUC__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wsign-conversion"
#endif

#include "chwd-cxxbridge/lib.h"

#if defined(__clang__)
#pragma clang diagnostic pop
#elif defined(__GNUC__)
#pragma GCC diagnostic pop
#endif

#include <optional>

namespace chwd {

using parse_profile_t = decltype(chwd::parse_profiles_ffi("", ""));
using vec_str_t = decltype(chwd::get_invalid_profiles_ffi(""));

auto parse_profiles(std::string_view file_path, std::string_view type) noexcept -> std::optional<parse_profile_t>;
auto get_invalid_profiles(std::string_view file_path) noexcept -> std::optional<vec_str_t> ;

}  // namespace chwd

#endif  // CONFIG_HPP
