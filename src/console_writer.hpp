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

#ifndef CONSOLE_WRITER_HPP
#define CONSOLE_WRITER_HPP

#include "config.hpp"
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

#include <string>
#include <string_view>
#include <vector>

namespace mhwd {

class ConsoleWriter {
 public:
    void print_status(const std::string_view& msg) const;
    void print_error(const std::string_view& msg) const;
    void print_warning(const std::string_view& msg) const;
    void print_message(mhwd::message_t type, const std::string_view& msg) const;
    void print_help() const;
    void print_version(const std::string_view& version, const std::string_view& year) const;
    void list_devices(const std::vector<std::shared_ptr<Device>>& devices,
        std::string typeOfDevice) const;
    void list_configs(const std::vector<std::shared_ptr<Config>>& configs,
        std::string header) const;
    void printAvailableConfigsInDetail(const std::string_view& deviceType,
        const std::vector<std::shared_ptr<Device>>& devices) const;
    void printInstalledConfigs(const std::string_view& deviceType,
        const std::vector<std::shared_ptr<Config>>& installedConfigs) const;
    void printConfigDetails(const Config& config) const;
    void printDeviceDetails(hw_item hw, FILE* f = stdout) const;

 private:
    void printLine() const;

    const char* CONSOLE_COLOR_RESET{"\033[m"};
    const char* CONSOLE_TEXT_OUTPUT_COLOR{"\033[0;32m"};
};

}  // namespace mhwd

#endif  // CONSOLE_WRITER_HPP
