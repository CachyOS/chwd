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

#include "mhwd.hpp"
#include "console_writer.hpp"
#include "const.hpp"
#include "vita/string.hpp"

#include <algorithm>   // for any_of
#include <filesystem>  // for exists, copy, remove_all
#include <stdexcept>   // for runtime_error
#include <string>      // for string
#include <vector>      // for vector

#if defined(__clang__)
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wold-style-cast"
#pragma clang diagnostic ignored "-Wimplicit-int-conversion"

#include <range/v3/algorithm/any_of.hpp>
#include <range/v3/algorithm/find_if.hpp>

#pragma clang diagnostic pop
#else
#include <ranges>
namespace ranges = std::ranges;
#endif

#include <fmt/compile.h>
#include <fmt/core.h>

namespace fs = std::filesystem;

namespace mhwd {

namespace {

inline bool is_user_root() noexcept {
    static constexpr auto ROOT_UID = 0;
    return ROOT_UID == getuid();
}

auto find_profile(const std::string_view& profile_name, const chwd::list_of_configs_t& profiles) noexcept -> chwd::profile_t {
    auto profile = ranges::find_if(profiles,
        [profile_name](const auto& temp) {
            return std::string(temp->name) == profile_name;
        });
    if (profile != profiles.end()) {
        return *profile;
    }
    return nullptr;
}

}  // namespace

bool Mhwd::perform_transaction(const chwd::profile_t& profile, chwd::Transaction transaction_type) {
    const auto transaction = Transaction{profile, transaction_type, m_arguments.force};
    const auto& status     = performTransaction(transaction);

    switch (status) {
    case chwd::Status::Success:
        break;
    case chwd::Status::ErrorNotInstalled:
        mhwd::console_writer::print_error("profile '{}' is not installed!", std::string(profile->name));
        break;
    case chwd::Status::ErrorAlreadyInstalled:
        mhwd::console_writer::print_warning("a version of profile '{}' is already installed!\nUse -f/--force to force installation...", std::string(profile->name));
        break;
    case chwd::Status::ErrorNoMatchLocalConfig:
        mhwd::console_writer::print_error("passed profile does not match with installed profile!");
        break;
    case chwd::Status::ErrorScriptFailed:
        mhwd::console_writer::print_error("script failed!");
        break;
    case chwd::Status::ErrorSetDatabase:
        mhwd::console_writer::print_error("failed to set database!");
        break;
    }

    m_data.update_installed_profile_data();

    return (chwd::Status::Success == status);
}

auto Mhwd::get_installed_profile(const std::string_view& profile_name, std::string_view config_type) const noexcept -> chwd::profile_t {
    // Get the right profiles
    const auto& installed_profiles = ("USB" == config_type) ? m_data.get_installed_usb_profiles() : m_data.get_installed_pci_profiles();
    return find_profile(profile_name, installed_profiles);
}

auto Mhwd::get_db_profile(const std::string_view& profile_name, std::string_view config_type) const noexcept -> chwd::profile_t {
    // Get the right profiles
    const auto& all_profiles = ("USB" == config_type) ? m_data.get_all_usb_profiles() : m_data.get_all_pci_profiles();
    return find_profile(profile_name, all_profiles);
}

auto Mhwd::get_available_profile(const std::string_view& config_name, std::string_view config_type) const noexcept -> chwd::profile_t {
    // Get the right devices
    const auto& devices = ("USB" == config_type) ? m_data.get_usb_devices() : m_data.get_pci_devices();

    for (const auto& device : devices) {
        const auto& available_profiles = device.get_available_profiles();
        if (available_profiles.empty()) {
            continue;
        }
        auto available_profile = ranges::find_if(available_profiles,
            [config_name](const auto& temp) {
                return std::string(temp.name) == config_name;
            });
        if (available_profile != available_profiles.end()) {
            return std::make_unique<chwd::Profile>(*available_profile);
        }
    }
    return nullptr;
}

auto Mhwd::performTransaction(const Transaction& transaction) -> chwd::Status {
    // Check if already installed
    const auto& installed_profile = get_installed_profile(std::string(transaction.profile->name), std::string(transaction.profile->prof_type));
    chwd::Status status           = chwd::Status::Success;

    if ((chwd::Transaction::Remove == transaction.type)
        || (installed_profile != nullptr && transaction.is_reinstall_allowed)) {
        if (nullptr == installed_profile) {
            return chwd::Status::ErrorNotInstalled;
        }
        mhwd::console_writer::print_message(chwd::Message::RemoveStart, std::string(installed_profile->name));
        status = uninstall_profile(installed_profile);
        if (chwd::Status::Success != status) {
            return status;
        }
        mhwd::console_writer::print_message(chwd::Message::RemoveEnd, std::string(installed_profile->name));
    }

    if (chwd::Transaction::Install == transaction.type) {
        // Check if already installed but not allowed to reinstall
        if ((nullptr != installed_profile) && !transaction.is_reinstall_allowed) {
            return chwd::Status::ErrorAlreadyInstalled;
        }
        mhwd::console_writer::print_message(chwd::Message::InstallStart, std::string(transaction.profile->name));
        status = install_profile(transaction.profile);
        if (chwd::Status::Success != status) {
            return status;
        }
        mhwd::console_writer::print_message(chwd::Message::InstallEnd, std::string(transaction.profile->name));
    }
    return status;
}

auto Mhwd::install_profile(const chwd::profile_t& profile) -> chwd::Status {
    if (!run_script(*profile, chwd::Transaction::Install)) {
        return chwd::Status::ErrorScriptFailed;
    }

    const auto& db_dir      = ("USB" == std::string(profile->prof_type)) ? consts::MHWD_USB_DATABASE_DIR : consts::MHWD_PCI_DATABASE_DIR;
    const auto& working_dir = fmt::format(FMT_COMPILE("{}/{}"), db_dir, fs::path{std::string(profile->prof_path)}.parent_path().filename().c_str());
    std::error_code err_code{};
    fs::create_directories(working_dir, err_code);
    if (!chwd::write_profile_to_file(fmt::format(FMT_COMPILE("{}/{}"), working_dir, consts::CHWD_CONFIG_FILE), *profile)) {
        return chwd::Status::ErrorSetDatabase;
    }

    // Note: installed profile vectors have to be updated manually with update_installed_profile_data(Data*)
    return chwd::Status::Success;
}

auto Mhwd::uninstall_profile(const chwd::profile_t& profile) noexcept -> chwd::Status {
    const auto& installed_profile = get_installed_profile(std::string(profile->name), std::string(profile->prof_type));

    // Check if installed
    if (nullptr == installed_profile) {
        return chwd::Status::ErrorNotInstalled;
    }
    // Run script
    if (!run_script(*installed_profile, chwd::Transaction::Remove)) {
        return chwd::Status::ErrorScriptFailed;
    }

    std::error_code err_code{};
    fs::remove(std::string(profile->prof_path), err_code);
    if (err_code.value() != 0) {
        return chwd::Status::ErrorSetDatabase;
    }

    m_data.update_installed_profile_data();
    return chwd::Status::Success;
}

bool Mhwd::run_script(const chwd::Profile& config, chwd::Transaction operation) noexcept {
    return chwd::run_script(m_data.get_raw_data(), config, operation);
}

auto Mhwd::tryToParseCmdLineOptions(std::span<char*> args, bool& autoconf_nonfree_driver, std::string& operation, std::string& autoconf_class_id) noexcept(false) -> std::int32_t {
    if (args.size() <= 1) {
        m_arguments.list_available = true;
    }
    const auto& proceed_install_option = [&operation](const auto& option, const auto& argument) {
        const std::string_view& device_type{argument};
        if (("pci" != device_type) && ("usb" != device_type)) {
            throw std::runtime_error{fmt::format(FMT_COMPILE("Invalid use of option: {}\n"), option)};
        }
        operation = Vita::string{device_type}.to_upper();
    };
    for (std::size_t nArg = 1; nArg < args.size(); ++nArg) {
        const std::string_view option{args[nArg]};
        if (("-h" == option) || ("--help" == option)) {
            mhwd::console_writer::print_help();
            return 1;
        } else if (("-v" == option) || ("--version" == option)) {
            mhwd::console_writer::print_version(m_version, m_year);
            return 1;
        } else if ("--is_nvidia_card" == option) {
            chwd::check_nvidia_card();
            return 1;
        } else if (("-f" == option) || ("--force" == option)) {
            m_arguments.force = true;
        } else if (("-d" == option) || ("--detail" == option)) {
            m_arguments.detail = true;
        } else if (("-la" == option) || ("--listall" == option)) {
            m_arguments.list_all = true;
        } else if (("-li" == option) || ("--listinstalled" == option)) {
            m_arguments.list_installed = true;
        } else if (("-l" == option) || ("--list" == option)) {
            m_arguments.list_available = true;
        } else if (("-lh" == option) || ("--listhardware" == option)) {
            m_arguments.list_hardware = true;
        } else if ("--pci" == option) {
            m_arguments.show_pci = true;
        } else if ("--usb" == option) {
            m_arguments.show_usb = true;
        } else if (("-a" == option) || ("--auto" == option)) {
            nArg += 3;
            if (nArg >= args.size()) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Too few arguments: {}\n"), option)};
            }
            const std::string_view& device_type{args[nArg - 2]};
            const std::string_view& driver_type{args[nArg - 1]};
            const std::string_view& class_id{args[nArg]};
            if ((("pci" != device_type) && ("usb" != device_type))
                || (("free" != driver_type) && ("nonfree" != driver_type))) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Invalid use of option: {}\n"), option)};
            }
            operation                 = Vita::string{device_type}.to_upper();
            autoconf_nonfree_driver   = ("nonfree" == driver_type);
            autoconf_class_id         = Vita::string{class_id}.to_lower().trim();
            m_arguments.autoconfigure = true;
        } else if (("-i" == option) || ("--install" == option)) {
            ++nArg;
            if (nArg >= args.size()) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Too few arguments: {}\n"), option)};
            }
            m_arguments.install = true;
            proceed_install_option(option, args[nArg]);
        } else if (("-r" == option) || ("--remove" == option)) {
            if ((nArg + 1) >= args.size()) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Too few arguments: {}\n"), option)};
            }
            const std::string_view& device_type{args[++nArg]};
            if (("pci" != device_type) && ("usb" != device_type)) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Invalid device type: {}\n"), device_type)};
            }
            operation          = Vita::string{device_type}.to_upper();
            m_arguments.remove = true;
        } else if ("--pmcachedir" == option) {
            if ((nArg + 1) >= args.size()) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Too few arguments: {}\n"), option)};
            }
            m_data.get_env_mut().pmcache_path = Vita::string(args[++nArg]).trim("\"").trim();
        } else if ("--pmconfig" == option) {
            if (nArg + 1 >= args.size()) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Too few arguments: {}\n"), option)};
            }
            m_data.get_env_mut().pmconfig_path = Vita::string(args[++nArg]).trim("\"").trim();
        } else if ("--pmroot" == option) {
            if (nArg + 1 >= args.size()) {
                throw std::runtime_error{fmt::format(FMT_COMPILE("Too few arguments: {}\n"), option)};
            }
            m_data.get_env_mut().pmroot_path = Vita::string(args[++nArg]).trim("\"").trim();
        } else if (m_arguments.install || m_arguments.remove) {
            const auto& name = Vita::string(args[nArg]).to_lower();
            if (!ranges::any_of(m_profiles, [name](auto&& config) { return name == config; })) {
                m_profiles.push_back(name);
            }
        } else {
            throw std::runtime_error{fmt::format(FMT_COMPILE("invalid option: {}\n"), args[nArg])};
        }
    }
    if (!m_arguments.show_pci && !m_arguments.show_usb) {
        m_arguments.show_usb = true;
        m_arguments.show_pci = true;
    }

    return 0;
}

void Mhwd::optionsDontInterfereWithEachOther() const noexcept(false) {
    if (m_arguments.install && m_arguments.remove) {
        throw std::runtime_error{"install and remove options can only be used separately!\n"};
    } else if ((m_arguments.install || m_arguments.remove) && m_arguments.autoconfigure) {
        throw std::runtime_error{"auto option can't be combined with install and remove options!\n"};
    } else if ((m_arguments.remove || m_arguments.install) && m_profiles.empty()) {
        throw std::runtime_error{"nothing to do?!\n"};
    }
}

auto Mhwd::launch(std::span<char*> args) -> std::int32_t {
    std::string operation{};
    bool autoconf_nonfree_driver{false};
    std::string autoconf_class_id{};

    try {
        const std::int32_t cmdline_code = tryToParseCmdLineOptions(args,
            autoconf_nonfree_driver, operation,
            autoconf_class_id);
        /* clang-format off */
        if (cmdline_code == 1) { return 0; }
        /* clang-format on */
        optionsDontInterfereWithEachOther();
    } catch (const std::runtime_error& e) {
        mhwd::console_writer::print_error(e.what());
        mhwd::console_writer::print_help();
        return 1;
    }

    m_data = chwd::Data::initialize_data();

    const auto& missing_dirs = chwd::check_environment();
    if (!missing_dirs.empty()) {
        mhwd::console_writer::print_error("Following directories do not exist:");
        for (const auto& missing_dir : missing_dirs) {
            mhwd::console_writer::print_status(std::string(missing_dir));
        }
        return 1;
    }

    // > Perform operations:
    chwd::handle_arguments_listing(m_data.get_raw_data(), m_arguments);

    // Auto configuration
    const auto& prepared_profiles = chwd::prepare_autoconfigure(m_data.get_raw_data(), m_arguments, operation.c_str(), autoconf_class_id.c_str(), autoconf_nonfree_driver);
    for (const auto& prepared_profile : prepared_profiles) {
        m_profiles.push_back(std::string(prepared_profile));
    }

    // Transaction
    /* clang-format off */
    if (!(m_arguments.install || m_arguments.remove)) { return 0; }
    /* clang-format on */
    if (!is_user_root()) {
        mhwd::console_writer::print_error("You cannot perform this operation unless you are root!");
        return 1;
    }
    for (auto&& profile_name : m_profiles) {
        if (m_arguments.install) {
            m_profile = get_available_profile(profile_name, operation);
            if (m_profile == nullptr) {
                m_profile = get_db_profile(profile_name, operation);
                if (m_profile == nullptr) {
                    mhwd::console_writer::print_error("profile '{}' does not exist!", profile_name);
                    return 1;
                }
                mhwd::console_writer::print_warning("no matching device for profile '{}' found!", profile_name);
            }

            if (!perform_transaction(m_profile, chwd::Transaction::Install)) {
                return 1;
            }
        } else if (m_arguments.remove) {
            m_profile = get_installed_profile(profile_name, operation);

            if (nullptr == m_profile) {
                mhwd::console_writer::print_error("profile '{}' is not installed!", profile_name);
                return 1;
            } else if (!perform_transaction(m_profile, chwd::Transaction::Remove)) {
                return 1;
            }
        }
    }
    return 0;
}

}  // namespace mhwd
