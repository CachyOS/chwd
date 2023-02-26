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

#ifndef TRANSACTION_HPP
#define TRANSACTION_HPP

#include "config.hpp"
#include "data.hpp"
#include "enums.hpp"

#include <memory>
#include <utility>
#include <vector>

namespace mhwd {

struct Transaction final {
    Transaction(const Data& data, std::shared_ptr<Config> conf, mhwd::transaction_t transaction_type, bool allow_reinstallation)
      : is_reinstall_allowed(allow_reinstallation),
        type(transaction_type), config(std::move(conf)),
        dependency_configs(data.getAllDependenciesToInstall(config)),
        conflicted_configs(data.getAllLocalConflicts(config)),
        configs_requirements(data.getAllLocalRequirements(config)) { }

    bool is_reinstall_allowed{};
    mhwd::transaction_t type{};

    std::shared_ptr<Config> config{};
    std::vector<std::shared_ptr<Config>> dependency_configs{};
    std::vector<std::shared_ptr<Config>> conflicted_configs{};
    std::vector<std::shared_ptr<Config>> configs_requirements{};

    // Deleted constructor
    Transaction() = delete;
};

}  // namespace mhwd

#endif  // TRANSACTION_HPP
