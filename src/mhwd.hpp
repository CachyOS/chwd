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

#include "config.hpp"
#include "console_writer.hpp"
#include "const.hpp"
#include "data.hpp"
#include "device.hpp"
#include "enums.hpp"
#include "transaction.hpp"

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
    struct Arguments {
        bool SHOW_PCI       = false;
        bool SHOW_USB       = false;
        bool INSTALL        = false;
        bool REMOVE         = false;
        bool DETAIL         = false;
        bool FORCE          = false;
        bool LIST_ALL       = false;
        bool LIST_INSTALLED = false;
        bool LIST_AVAILABLE = false;
        bool LIST_HARDWARE  = false;
        bool AUTOCONFIGURE  = false;
    } m_arguments;

    std::string_view m_version{};
    std::string_view m_year{};
    std::shared_ptr<chwd::Profile> m_config;
    Data m_data;
    std::vector<std::string> m_configs{};

    bool performTransaction(const profile_t& config, mhwd::transaction_t type);

    [[nodiscard]] auto getInstalledConfig(const std::string_view& config_name, std::string_view config_type) const noexcept -> profile_t;
    [[nodiscard]] auto getDatabaseConfig(const std::string_view& config_name, std::string_view config_type) const noexcept -> profile_t;
    [[nodiscard]] auto getAvailableConfig(const std::string_view& config_name, std::string_view config_type) const noexcept -> profile_t;

    auto performTransaction(const Transaction& transaction) -> mhwd::status_t;

    auto installConfig(const profile_t& config) -> mhwd::status_t;
    auto uninstallConfig(chwd::Profile* config) noexcept -> mhwd::status_t;
    bool runScript(const profile_t& config, mhwd::transaction_t operation) noexcept;
    auto tryToParseCmdLineOptions(std::span<char*> args, bool& autoconf_nonfree_driver,
        std::string& operation, std::string& autoconf_class_id) noexcept(false) -> std::int32_t;
    void optionsDontInterfereWithEachOther() const noexcept(false);
    void checkNvidiaCard() noexcept;
};

}  // namespace mhwd

#endif  // MHWD_HPP
