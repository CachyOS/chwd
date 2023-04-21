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

#include "data.hpp"

#include <vector>

namespace chwd {

auto Data::initialize_data() noexcept -> Data {
    Data res;
    res.m_handle = chwd::initialize_data_obj();
    return res;
}

void Data::update_installed_profile_data() noexcept {
    m_handle->update_installed_profile_data();
}

auto Data::get_all_pci_profiles() const noexcept -> list_of_configs_t {
    const auto& pci_profiles = m_handle->get_all_pci_profiles();
    list_of_configs_t configs;

    for (const auto& pci_profile : pci_profiles) {
        configs.push_back(std::make_unique<Profile>(pci_profile));
    }
    return configs;
}
auto Data::get_all_usb_profiles() const noexcept -> list_of_configs_t {
    const auto& usb_profiles = m_handle->get_all_usb_profiles();
    list_of_configs_t configs;

    for (const auto& usb_profile : usb_profiles) {
        configs.push_back(std::make_unique<Profile>(usb_profile));
    }
    return configs;
}
auto Data::get_installed_pci_profiles() const noexcept -> list_of_configs_t {
    const auto& pci_profiles = m_handle->get_installed_pci_profiles();
    list_of_configs_t configs;

    for (const auto& pci_profile : pci_profiles) {
        configs.push_back(std::make_unique<Profile>(pci_profile));
    }
    return configs;
}
auto Data::get_installed_usb_profiles() const noexcept -> list_of_configs_t {
    const auto& usb_profiles = m_handle->get_installed_usb_profiles();
    list_of_configs_t configs;

    for (const auto& usb_profile : usb_profiles) {
        configs.push_back(std::make_unique<Profile>(usb_profile));
    }
    return configs;
}
auto Data::get_pci_devices() const noexcept -> list_of_devices_t const& {
    return m_handle->get_pci_devices();
}
auto Data::get_usb_devices() const noexcept -> list_of_devices_t const& {
    return m_handle->get_usb_devices();
}

auto Data::get_env_mut() noexcept -> chwd::Environment& {
    return m_handle->get_env_mut();
}

}  // namespace chwd
