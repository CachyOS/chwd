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

#ifndef MHWD_HPP
#define MHWD_HPP

#include "data.hpp"
#include "transaction.hpp"

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
#include <span>         // for span
#include <string>       // for string
#include <string_view>  // for string_view
#include <vector>       // for vector

namespace mhwd {

class Mhwd {
 public:
    Mhwd(const std::string_view& version, const std::string_view& year) : m_version(version), m_year(year) { }
    ~Mhwd() = default;
    int launch(std::span<char*> args);

 private:
    chwd::Arguments m_arguments{};

    std::string_view m_version{};
    std::string_view m_year{};
    std::shared_ptr<chwd::Profile> m_profile;
    chwd::Data m_data;
    std::vector<std::string> m_profiles{};

    bool perform_transaction(const chwd::profile_t& profile, chwd::Transaction transaction_type);

    [[nodiscard]] auto get_installed_profile(const std::string_view& config_name, std::string_view config_type) const noexcept -> chwd::profile_t;
    [[nodiscard]] auto get_db_profile(const std::string_view& profile_name, std::string_view config_type) const noexcept -> chwd::profile_t;
    [[nodiscard]] auto get_available_profile(const std::string_view& config_name, std::string_view config_type) const noexcept -> chwd::profile_t;

    auto performTransaction(const Transaction& transaction) -> chwd::Status;

    auto install_profile(const chwd::profile_t& profile) -> chwd::Status;
    auto uninstall_profile(const chwd::profile_t& profile) noexcept -> chwd::Status;
    bool run_script(const chwd::Profile& profile, chwd::Transaction operation) noexcept;
    auto tryToParseCmdLineOptions(std::span<char*> args, bool& autoconf_nonfree_driver,
        std::string& operation, std::string& autoconf_class_id) noexcept(false) -> std::int32_t;
    void optionsDontInterfereWithEachOther() const noexcept(false);
};

}  // namespace mhwd

#endif  // MHWD_HPP
