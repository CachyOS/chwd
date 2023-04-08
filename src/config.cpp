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

#include "config.hpp"
#include "utils.hpp"

#include <algorithm>  // for transform
#include <fstream>    // for ifstream
#include <string>     // for string
#include <vector>     // for vector

#include <fmt/compile.h>
#include <fmt/core.h>
#include <fmt/ranges.h>

#define TOML_EXCEPTIONS 0  // disable exceptions
#include <toml++/toml.h>

#if defined(__clang__)
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wold-style-cast"
#pragma clang diagnostic ignored "-Wimplicit-int-conversion"

#include <range/v3/algorithm/for_each.hpp>
#include <range/v3/view/filter.hpp>
#include <range/v3/view/split.hpp>
#include <range/v3/view/transform.hpp>

#pragma clang diagnostic pop
#else
#include <ranges>
namespace ranges = std::ranges;
#endif

namespace mhwd {

namespace {
auto split_str(std::string_view str, std::uint8_t delim = ' ') noexcept -> std::vector<std::string> {
    static constexpr auto functor = [](auto&& rng) {
        return std::string_view(&*rng.begin(), static_cast<size_t>(ranges::distance(rng)));
    };
    static constexpr auto second = [](auto&& rng) { return rng != ""; };

    /* clang-format off */
    auto&& view_res = str
        | ranges::views::split(delim)
        | ranges::views::transform(functor);
    /* clang-format on */

    std::vector<std::string> lines{};
    ranges::for_each(view_res | ranges::views::filter(second), [&](auto&& rng) { lines.emplace_back(rng); });
    return lines;
}

auto join_vec(const std::vector<std::string>& vec, char delim) noexcept -> std::string {
    std::string joined_str{};
    ranges::for_each(vec, [&](auto&& rng) { joined_str += fmt::format("{}{}", rng, delim); });
    if (joined_str.ends_with(delim)) {
        joined_str.pop_back();
    }
    return joined_str;
}

void merge_table_left(toml::table& lhs, toml::table&& rhs) {
    rhs.for_each(
        [&](const toml::key& rhs_key, auto&& rhs_val) {
            auto lhs_it = lhs.lower_bound(rhs_key);

            // rhs key not found in lhs - direct move
            if (lhs_it == lhs.cend() || lhs_it->first != rhs_key) {
                using rhs_type = std::remove_cv_t<std::remove_reference_t<decltype(rhs_val)>>;
                lhs.emplace_hint<rhs_type>(lhs_it, rhs_key, std::forward<decltype(rhs_val)>(rhs_val));
                return;
            }
        });
}
}  // namespace

auto parse_ids_file(std::string_view file_path) noexcept -> std::optional<Vita::string> {
    using namespace std::string_view_literals;

    std::ifstream extern_file(file_path.data());
    /* clang-format off */
    if (!extern_file.is_open()) { return {}; }
    /* clang-format on */

    Vita::string ret{};

    while (!extern_file.eof()) {
        Vita::string line{};
        std::getline(extern_file, line);

        const auto pos = line.find_first_of('#');
        /* clang-format off */
        if (std::string::npos != pos) { line.erase(pos); }
        if (line.trim().empty()) { continue; }
        /* clang-format on */

        ret += fmt::format(FMT_COMPILE(" {}"), line.trim());
    }

    ret = ret.trim();

    // remove all multiple spaces
    while (ret.find("  "sv) != std::string::npos) {
        ret = ret.replace("  "sv, " "sv);
    }

    return ret;
}

auto parse_profile(auto&& node, std::string_view profile_name) noexcept -> std::optional<Profile> {
    using namespace std::string_view_literals;
    Profile profile;

    const auto& conf_name      = profile_name;
    const auto& conf_desc      = node["desc"sv].value_or(""sv);
    const auto& conf_nonfree   = node["nonfree"sv].value_or(false);
    const auto& conf_devids    = node["device_ids"sv].value_or(""sv);
    const auto& conf_vendorids = node["vendor_ids"sv].value_or(""sv);
    const auto& conf_classids  = node["class_ids"sv].value_or(""sv);
    const auto& conf_priority  = node["priority"sv].value_or(0);

    profile.name       = conf_name.data();
    profile.desc       = conf_desc.data();
    profile.is_nonfree = conf_nonfree;
    profile.priority   = conf_priority;

    // Read ids in extern file
    Vita::string devids_val{};
    if (!conf_devids.empty() && conf_devids.front() == '>') {
        const auto& parsed_ids = parse_ids_file(conf_devids.substr(1));
        /* clang-format off */
        if (!parsed_ids) { return {}; }
        /* clang-format on */
        devids_val = parsed_ids.value();
    }

    // Add new HardwareIDs group to vector if vector is not empty
    if (!profile.hwd_ids.back().device_ids.empty()) {
        profile.hwd_ids.emplace_back();
    }
    profile.hwd_ids.back().device_ids = split_str(devids_val);
    if (!profile.hwd_ids.back().class_ids.empty()) {
        profile.hwd_ids.emplace_back();
    }
    profile.hwd_ids.back().class_ids = split_str(conf_classids);

    if (!conf_vendorids.empty()) {
        // Add new HardwareIDs group to vector if vector is not empty
        if (!profile.hwd_ids.back().vendor_ids.empty()) {
            profile.hwd_ids.emplace_back();
        }
        profile.hwd_ids.back().vendor_ids = split_str(conf_vendorids);
    }

    const auto& append_star = [](auto& vec) { if (vec.empty()) { vec.emplace_back("*"sv); } };

    // Append * to all empty vectors
    for (auto& hwd_id : profile.hwd_ids) {
        append_star(hwd_id.class_ids);
        append_star(hwd_id.vendor_ids);
        append_star(hwd_id.device_ids);
    }
    return profile;
}

auto Profile::parse_profiles(const std::string_view& file_path, std::string_view type_name) noexcept -> std::optional<std::vector<Profile>> {
    toml::parse_result config = toml::parse_file(file_path);
    /* clang-format off */
    if (config.failed()) { return {}; }
    /* clang-format on */

    std::vector<Profile> profile_list{};
    for (auto&& [key, value] : config.table()) {
        if (value.is_table()) {
            auto toplevel_profile = parse_profile(*value.as_table(), key);
            /* clang-format off */
            if (!toplevel_profile) { continue; }
            /* clang-format on */
            for (auto&& [nested_key, nested_value] : *value.as_table()) {
                /* clang-format off */
                if (!nested_value.is_table()) { continue; }
                /* clang-format on */

                const auto& nested_profile_name = fmt::format("{}.{}", key, nested_key);
                auto nested_value_table         = *nested_value.as_table();
                merge_table_left(nested_value_table, std::move(*value.as_table()));
                auto nested_profile = parse_profile(nested_value_table, nested_profile_name);
                /* clang-format off */
                if (!nested_profile) { continue; }
                /* clang-format on */

                nested_profile.value().type = type_name;
                profile_list.push_back(nested_profile.value());
            }
            toplevel_profile.value().type = type_name;
            profile_list.push_back(toplevel_profile.value());
        }
    }
    return profile_list;
}

auto Profile::get_invalid_profiles(const std::string_view& file_path) noexcept -> std::optional<std::vector<std::string>> {
    toml::parse_result config = toml::parse_file(file_path);
    /* clang-format off */
    if (config.failed()) { return {}; }
    /* clang-format on */

    std::vector<std::string> invalid_profile_list{};
    for (auto&& [key, value] : config.table()) {
        if (value.is_table()) {
            auto toplevel_profile = parse_profile(*value.as_table(), key);
            if (!toplevel_profile) {
                invalid_profile_list.emplace_back(key.str().data());
                continue;
            }
            for (auto&& [nested_key, nested_value] : *value.as_table()) {
                /* clang-format off */
                if (!nested_value.is_table()) { continue; }
                /* clang-format on */

                const auto& nested_profile_name = fmt::format("{}.{}", key, nested_key);
                auto nested_value_table         = *nested_value.as_table();
                merge_table_left(nested_value_table, std::move(*value.as_table()));
                auto nested_profile = parse_profile(nested_value_table, nested_profile_name);
                /* clang-format off */
                if (nested_profile) { continue; }
                /* clang-format on */
                invalid_profile_list.push_back(nested_profile_name);
            }
        }
    }
    return invalid_profile_list;
}

auto Profile::write_profile_to_file(const std::string_view& file_path, const Profile& profile) noexcept -> bool {
    auto table = toml::table{
        {"nonfree", profile.is_nonfree},
        {"desc", profile.desc},
        {"priority", profile.priority},
    };

    const auto& device_ids = profile.hwd_ids.back().device_ids;
    const auto& vendor_ids = profile.hwd_ids.back().vendor_ids;
    const auto& class_ids  = profile.hwd_ids.back().class_ids;
    table.insert("device_ids", join_vec(device_ids, ' '));
    table.insert("vendor_ids", join_vec(vendor_ids, ' '));
    table.insert("class_ids", join_vec(class_ids, ' '));

    std::stringstream buf1;
    buf1 << table;
    const auto& toml_string = fmt::format("[{}]\n{}", profile.name, buf1.str());
    std::ofstream out_file{file_path.data()};
    /* clang-format off */
    if (!out_file.is_open()) { return false; }
    /* clang-format on */
    out_file << toml_string;
    return true;
}

}  // namespace mhwd
