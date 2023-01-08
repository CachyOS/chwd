//
//  mhwd - Manjaro Hardware Detection
//  Roland Singer <roland@manjaro.org>
//
//  Copyright (C) 2007 Free Software Foundation, Inc.
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.

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

#ifndef CONST_HPP
#define CONST_HPP

namespace mhwd {
namespace consts {

    static constexpr auto MHWD_CONFIG_NAME      = "MHWDCONFIG";
    static constexpr auto MHWD_USB_CONFIG_DIR   = "/var/lib/mhwd/db/usb";
    static constexpr auto MHWD_PCI_CONFIG_DIR   = "/var/lib/mhwd/db/pci";
    static constexpr auto MHWD_USB_DATABASE_DIR = "/var/lib/mhwd/local/usb";
    static constexpr auto MHWD_PCI_DATABASE_DIR = "/var/lib/mhwd/local/pci";
    static constexpr auto MHWD_SCRIPT_PATH      = "/var/lib/mhwd/scripts/mhwd";

    static constexpr auto MHWD_PM_CACHE_DIR = "/var/cache/pacman/pkg";
    static constexpr auto MHWD_PM_CONFIG    = "/etc/pacman.conf";
    static constexpr auto MHWD_PM_ROOT      = "/";

}  // namespace consts
}  // namespace mhwd

#endif  // CONST_HPP
