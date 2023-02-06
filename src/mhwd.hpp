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

#include <dirent.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

#include <cstdio>
#include <cstdlib>
#include <memory>
#include <string>
#include <string_view>
#include <vector>

#include "vita/string.hpp"

namespace mhwd {

class Mhwd {
 public:
    Mhwd(const std::string_view& ver, const std::string_view& year) : m_version(ver), m_year(year) { }
    ~Mhwd() = default;
    int launch(int argc, char** argv);

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
        bool CUSTOM_INSTALL = false;
        bool AUTOCONFIGURE  = false;
    } m_arguments;

    const std::string_view m_version{};
    const std::string_view m_year{};
    std::shared_ptr<Config> m_config;
    Data m_data;
    ConsoleWriter m_console_writer{};
    std::vector<std::string> m_configs{};

    bool performTransaction(const std::shared_ptr<Config>& config, mhwd::transaction_t type);

    [[nodiscard]] std::shared_ptr<Config> getInstalledConfig(const std::string_view& configName, const std::string_view& configType) const noexcept;
    [[nodiscard]] std::shared_ptr<Config> getDatabaseConfig(const std::string_view& configName, const std::string_view& configType) const noexcept;
    [[nodiscard]] std::shared_ptr<Config> getAvailableConfig(const std::string_view& configName, const std::string_view& configType) const noexcept;

    mhwd::status_t performTransaction(const Transaction& transaction);

    mhwd::status_t installConfig(const std::shared_ptr<Config>& config);
    mhwd::status_t uninstallConfig(Config* config) noexcept;
    bool runScript(const std::shared_ptr<Config>& config, mhwd::transaction_t operationType) noexcept;
    std::int32_t tryToParseCmdLineOptions(int argc, char* argv[], bool& autoConfigureNonFreeDriver,
        std::string& operationType, std::string& autoConfigureClassID);
    void optionsDontInterfereWithEachOther() const;
    void checkNvidiaCard() noexcept;
};

}  // namespace mhwd

#endif  // MHWD_HPP
