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

#include "kernel.hpp"
#include "ini.hpp"
#include "utils.hpp"

#include <fmt/compile.h>
#include <fmt/core.h>

#if defined(__clang__)
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wold-style-cast"

#include <range/v3/algorithm/search.hpp>

#pragma clang diagnostic pop
#else
#include <algorithm>
#include <ranges>
namespace ranges = std::ranges;
#endif

namespace {

void parse_cachedirs(alpm_handle_t* handle) noexcept {
    static constexpr auto cachedir = "/var/cache/pacman/pkg/";

    alpm_list_t* cachedirs = nullptr;
    cachedirs              = alpm_list_add(cachedirs, const_cast<void*>(reinterpret_cast<const void*>(cachedir)));
    alpm_option_set_cachedirs(handle, cachedirs);
}

void parse_includes(alpm_handle_t* handle, alpm_db_t* db, const auto& section, const auto& file) noexcept {
    const auto* archs = alpm_option_get_architectures(handle);
    const auto* arch  = reinterpret_cast<const char*>(archs->data);

    mINI::INIFile file_nested(file);
    // next, create a structure that will hold data
    mINI::INIStructure mirrorlist;

    // now we can read the file
    file_nested.read(mirrorlist);
    for (const auto& mirror : mirrorlist) {
        auto repo = mirror.second.begin()->second;
        if (repo.starts_with("/")) {
            continue;
        }
        utils::replace_all(repo, "$arch", arch);
        utils::replace_all(repo, "$repo", section.c_str());
        alpm_db_add_server(db, repo.c_str());
    }
}

void parse_repos(alpm_handle_t* handle) noexcept {
    static constexpr auto pacman_conf_path = "/etc/pacman.conf";
    static constexpr auto ignored_repo     = "testing";

    mINI::INIFile file(pacman_conf_path);
    // next, create a structure that will hold data
    mINI::INIStructure ini;

    // now we can read the file
    file.read(ini);
    for (const auto& it : ini) {
        const auto& section = it.first;
        const auto& nested  = it.second;
        if (section == ignored_repo) {
            continue;
        }
        if (section == "options") {
            for (const auto& it_nested : nested) {
                if (it_nested.first != "architecture") {
                    continue;
                }
                // add CacheDir
                const auto& archs = utils::make_multiline(it_nested.second, false, " ");
                for (const auto& arch : archs) {
                    alpm_option_add_architecture(handle, arch.c_str());
                }
            }
            continue;
        }
        auto* db = alpm_register_syncdb(handle, section.c_str(), ALPM_SIG_USE_DEFAULT);

        for (const auto& it_nested : nested) {
            const auto& param = it_nested.first;
            const auto& value = it_nested.second;
            if (param == "include") {
                parse_includes(handle, db, section, value);
            }
        }
    }
}
}  // namespace

std::string Kernel::version() const noexcept {
    const char* sync_pkg_ver = alpm_pkg_get_version(m_pkg);
    if (!is_installed()) {
        return sync_pkg_ver;
    }

    auto* db        = alpm_get_localdb(m_handle);
    auto* local_pkg = alpm_db_get_pkg(db, m_name.c_str());
    return alpm_pkg_get_version(local_pkg);
}

// Name must be without any repo name (e.g. core/linux)
bool Kernel::is_installed() const noexcept {
    auto* db  = alpm_get_localdb(m_handle);
    auto* pkg = alpm_db_get_pkg(db, m_name.c_str());

    return pkg != NULL;
}

// Find kernel packages by finding packages which have words 'linux' and 'headers'.
// From the output of 'pacman -Sl'
// - find lines that have words: 'linux' and 'headers'
// - drop lines containing 'testing' (=testing repo, causes duplicates) and 'linux-api-headers' (=not a kernel header)
// - show the (header) package names
// Now we have names of the kernel headers.
// Then add the kernel packages to proper places and output the result.
// Then display possible kernels and headers added by the user.

// The output consists of a list of reponame and a package name formatted as: "reponame/pkgname"
// For example:
//    reponame/linux-xxx reponame/linux-xxx-headers
//    reponame/linux-yyy reponame/linux-yyy-headers
//    ...
std::vector<Kernel> Kernel::get_kernels(alpm_handle_t* handle) noexcept {
    static constexpr std::string_view ignored_pkg  = "linux-api-headers";
    static constexpr std::string_view replace_part = "-headers";
    std::vector<Kernel> kernels{};

    parse_repos(handle);
    parse_cachedirs(handle);

    auto* dbs = alpm_get_syncdbs(handle);
    for (alpm_list_t* i = dbs; i != nullptr; i = i->next) {
        static constexpr auto needle = "linux[^ ]*-headers";
        alpm_list_t* needles         = nullptr;
        alpm_list_t* ret_list        = nullptr;
        needles                      = alpm_list_add(needles, const_cast<void*>(reinterpret_cast<const void*>(needle)));

        auto* db            = reinterpret_cast<alpm_db_t*>(i->data);
        const char* db_name = alpm_db_get_name(db);
        alpm_db_search(db, needles, &ret_list);

        for (alpm_list_t* j = ret_list; j != nullptr; j = j->next) {
            auto* pkg            = reinterpret_cast<alpm_pkg_t*>(j->data);
            std::string pkg_name = alpm_pkg_get_name(pkg);
            const auto& found    = ranges::search(pkg_name, ignored_pkg);
            if (!found.empty()) {
                continue;
            }
            utils::remove_all(pkg_name, replace_part);
            pkg = alpm_db_get_pkg(db, pkg_name.c_str());

            // Skip if the actual kernel package is not found
            if (!pkg) {
                continue;
            }

            kernels.emplace_back(Kernel{handle, pkg, db_name, fmt::format(FMT_COMPILE("{}/{}"), db_name, pkg_name)});
        }

        alpm_list_free(needles);
        alpm_list_free(ret_list);
    }

    return kernels;
}
