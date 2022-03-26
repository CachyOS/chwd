// Copyright (C) 2022 Vladislav Nepogodin
//
// This file is part of CachyOS kernel manager.
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

#ifndef KERNEL_HPP
#define KERNEL_HPP

#include <string>
#include <string_view>
#include <vector>

#include <alpm.h>

class Kernel {
 public:
    consteval Kernel() = default;
    explicit Kernel(alpm_handle_t* handle, alpm_pkg_t* pkg) : m_name(alpm_pkg_get_name(pkg)), m_pkg(pkg), m_handle(handle) { }
    explicit Kernel(alpm_handle_t* handle, alpm_pkg_t* pkg, const std::string_view& repo) : m_name(alpm_pkg_get_name(pkg)), m_repo(repo), m_pkg(pkg), m_handle(handle) { }
    explicit Kernel(alpm_handle_t* handle, alpm_pkg_t* pkg, const std::string_view& repo, const std::string_view& raw) : m_name(alpm_pkg_get_name(pkg)), m_repo(repo), m_raw(raw), m_pkg(pkg), m_handle(handle) { }

    [[nodiscard]] std::string version() const noexcept;

    [[nodiscard]] bool is_installed() const noexcept;

    [[nodiscard]] inline const char* name() const noexcept
    { return m_name.c_str(); }

    [[nodiscard]] inline const char* get_raw() const noexcept
    { return m_raw.c_str(); }
    /* clang-format on */

    static std::vector<Kernel> get_kernels(alpm_handle_t* handle) noexcept;

 private:
    std::string m_name{};
    std::string m_repo{"local"};
    std::string m_raw{};

    alpm_pkg_t* m_pkg;
    alpm_handle_t* m_handle;
};

#endif  // KERNEL_HPP
