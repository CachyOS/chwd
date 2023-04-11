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

#ifndef CONSOLE_WRITER_HPP
#define CONSOLE_WRITER_HPP

#include "config.hpp"
#include "data.hpp"
#include "device.hpp"
#include "enums.hpp"

#if defined(__clang__)
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wnested-anon-types"
#endif

#include <hd.h>

#if defined(__clang__)
#pragma clang diagnostic pop
#endif

#include <string_view>

#include <fmt/core.h>

namespace mhwd::console_writer {

void print_status(const std::string_view& msg);
void print_error(const std::string_view& msg);
void print_warning(const std::string_view& msg);

// overload to allow pass variadic arguments
template <typename... Args>
constexpr void print_status(fmt::format_string<Args...> fmt, Args&&... args) {
    print_status(fmt::format(fmt, std::forward<Args>(args)...));
}
template <typename... Args>
constexpr void print_error(fmt::format_string<Args...> fmt, Args&&... args) {
    print_error(fmt::format(fmt, std::forward<Args>(args)...));
}
template <typename... Args>
constexpr void print_warning(fmt::format_string<Args...> fmt, Args&&... args) {
    print_warning(fmt::format(fmt, std::forward<Args>(args)...));
}

void print_message(mhwd::message_t type, const std::string_view& msg);
void list_devices(const list_of_devices_t& devices, const std::string_view& type_of_device);
void list_configs(const list_of_configs_t& configs, const std::string_view& header);
void printAvailableConfigsInDetail(const std::string_view& device_type, const list_of_devices_t& devices);
void printInstalledConfigs(const std::string_view& device_type, const list_of_configs_t& installed_configs);
void printConfigDetails(const chwd::Profile& config) noexcept;
void printDeviceDetails(hw_item item, FILE* file_obj = stdout) noexcept;

void print_help() noexcept;
void print_version(const std::string_view& version_mhwd, const std::string_view& year_copy) noexcept;

}  // namespace mhwd::console_writer

#endif  // CONSOLE_WRITER_HPP
