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

#ifndef DATA_HPP
#define DATA_HPP

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

#include <memory>       // for shared_ptr
#include <string_view>  // for string_view
#include <vector>       // for vector

namespace chwd {

using profile_t         = std::shared_ptr<chwd::Profile>;
using list_of_configs_t = std::vector<profile_t>;
using list_of_devices_t = rust::Vec<chwd::DeviceFFi>;

class Data final {
 public:
    Data()  = default;
    ~Data() = default;

    Data(const Data& other) noexcept = delete;
    Data(Data&& other) noexcept : m_handle(std::move(other.m_handle)) { }

    Data& operator=(const Data& other) = delete;
    Data& operator=(Data&& other) {
        m_handle = std::move(other.m_handle);
        return *this;
    }

    static auto initialize_data() noexcept -> Data;

    auto get_all_pci_profiles() const noexcept -> list_of_configs_t;
    auto get_all_usb_profiles() const noexcept -> list_of_configs_t;
    auto get_installed_pci_profiles() const noexcept -> list_of_configs_t;
    auto get_installed_usb_profiles() const noexcept -> list_of_configs_t;
    auto get_pci_devices() const noexcept -> list_of_devices_t const&;
    auto get_usb_devices() const noexcept -> list_of_devices_t const&;
    auto get_env_mut() noexcept -> chwd::Environment&;

    auto get_raw_data() noexcept -> rust::Box<chwd::DataFFi>& { return m_handle; }

    void update_installed_profile_data() noexcept;

 private:
    rust::Box<chwd::DataFFi> m_handle = rust::Box<chwd::DataFFi>::from_raw(nullptr);
};

}  // namespace chwd

#endif  // DATA_HPP
