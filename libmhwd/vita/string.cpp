/**
 * @file string.cpp
 * Vita::string definition.
 *
 * Licensed under the terms of the MIT/X11 license.
 * Copyright (c) 2008 Vita Smid <me@ze.phyr.us>
 *
 * $Id: string.cpp 17 2008-08-11 17:46:13Z zephyr $
 */

#include "string.hpp"

#include <algorithm>

#if defined(__clang__)
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wold-style-cast"
#pragma clang diagnostic ignored "-Wimplicit-int-conversion"

#include <range/v3/algorithm/transform.hpp>

#pragma clang diagnostic pop
#else
#include <ranges>
namespace ranges = std::ranges;
#endif

namespace Vita {

string string::toLower() const {
    string result;
    ranges::transform(
        *this, std::back_inserter(result),
        [](auto c) -> auto { return std::tolower(c); });
    return result;
}

string string::toUpper() const {
    string result;
    ranges::transform(
        *this, std::back_inserter(result),
        [](auto c) -> auto { return std::toupper(c); });
    return result;
}

string string::ucfirst() const {
    string result = *this;
    result[0]     = static_cast<int8_t>(std::toupper(result[0]));
    return result;
}

string string::lcfirst() const {
    string result = *this;
    result[0]     = static_cast<int8_t>(std::tolower(result[0]));
    return result;
}

string string::operator+(int operand) const {
    return (*this) + string::toStr<int>(operand);
}

string string::operator+(long int operand) const {
    return (*this) + string::toStr<long int>(operand);
}

string string::operator+(double operand) const {
    return (*this) + string::toStr<double>(operand);
}

string string::operator+(float operand) const {
    return (*this) + string::toStr<float>(operand);
}

string string::replace(const std::string_view& search, const std::string_view& replace, size_t limit) const {
    const std::string_view tmp = *this;
    string result;
    size_t previous = 0;

    size_t current = tmp.find(search);

    while ((current != std::string_view::npos) && limit) {
        result += tmp.substr(previous, current - previous);
        result += replace;
        previous = current + search.size();
        current  = tmp.find(search, previous);
        --limit;
    }
    result += tmp.substr(previous);
    return result;
}

std::vector<string> string::explode(const std::string_view& delimiter) const {
    if (empty())
        return {};

    std::vector<string> result;
    size_t previous = 0;

    size_t current = find(delimiter);

    while (current != std::string::npos) {
        result.emplace_back(substr(previous, current - previous));
        previous = current + delimiter.size();
        current  = find(delimiter, previous);
    }
    result.emplace_back(substr(previous));
    return result;
}

string string::trim(const std::string_view& what) const {
    string result = *this;
    size_t pos    = result.find_first_not_of(what);
    result.erase(0, pos);
    pos = result.find_last_not_of(what);
    result.erase(pos + 1);
    return result;
}

}  // namespace Vita
