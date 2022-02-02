/*
 *  This file is part of the mhwd - Manjaro Hardware Detection project
 *
 *  mhwd - Manjaro Hardware Detection
 *  Roland Singer <roland@manjaro.org>
 *  Łukasz Matysiak <december0123@gmail.com>
 *  Filipe Marques <eagle.software3@gmail.com>
 *  Oscar Forner Martinez <oscar.forner.martinez@gmail.com>
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
// Copyright (C) 2022 Vladislav Nepogodin
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

#ifndef UTILS_HPP
#define UTILS_HPP

#include <cstdint>

namespace mhwd {
namespace utils {

    using hash_t = std::uint64_t;

    // Hardcoded values for the prime and the basis to calculate hash values.
    static constexpr hash_t prime = 0x100000001B3ull;
    static constexpr hash_t basis = 0xCBF29CE484222325ull;

    consteval hash_t hash_compile_time(const char* str, hash_t last_value = basis) {
        return (*str) ? hash_compile_time(str + 1, (static_cast<hash_t>(*str) ^ last_value) * prime) : last_value;
    }

    constexpr hash_t hash(const char* str) {
        hash_t result{basis};

        while (str != nullptr) {
            result ^= static_cast<hash_t>(*str);
            result *= prime;
            ++str;
        }

        return result;
    }

}  // namespace utils
}  // namespace mhwd

#endif  // UTILS_HPP
