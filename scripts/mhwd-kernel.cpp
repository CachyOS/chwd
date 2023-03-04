// Copyright (C) 2022-2023 Vladislav Nepogodin
//
// This file is part of CachyOS Mhwd.
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

#include <unistd.h>

#include <cstdlib>
#include <iostream>
#include <memory>
#include <span>
#include <string_view>

#if defined(__clang__)
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wold-style-cast"

#include <range/v3/algorithm/any_of.hpp>
#include <range/v3/algorithm/none_of.hpp>

#pragma clang diagnostic pop
#else
#include <algorithm>
#include <ranges>
namespace ranges = std::ranges;
#endif

#include <fmt/core.h>
#include <optional>

namespace {
class AlpmHelper final {
 public:
    AlpmHelper() = default;
    ~AlpmHelper() {
        alpm_release(m_handle);
    }

    [[nodiscard]] const std::vector<Kernel>& kernels() const noexcept {
        return m_kernels;
    }

 private:
    alpm_errno_t m_err{};
    alpm_handle_t* m_handle       = alpm_initialize("/", "/var/lib/pacman/", &m_err);
    std::vector<Kernel> m_kernels = Kernel::get_kernels(m_handle);
};

[[noreturn]] void err(const std::string_view& msg) noexcept {
    fmt::print(stderr, "\033[31mError:\033[0m {}\n", msg);
    std::exit(1);
}

void root_check() noexcept {
    /* clang-format off */
    if (geteuid() == 0) { return; }
    /* clang-format on */
    err("Please run as root.");
}

void kernel_usage() noexcept {
    std::cout <<
        R"(Usage: mhwd-kernel [option]
    -h  --help              Show this help message
    -i  --install           Install a new kernel        [kernel(s)] [optional: rmc = remove current kernel]
    -l  --list              List all available kernels
    -li --listinstalled     List installed kernels
    -lr --listrunning       List running kernel
    -r  --remove            Remove a kernel             [kernel(s)])"
              << '\n';
}

std::string exec(const std::string_view& command) noexcept {
    std::unique_ptr<FILE, decltype(&pclose)> pipe(popen(command.data(), "r"), pclose);
    /* clang-format off */
    if (!pipe) { return "-1"; }
    /* clang-format on */

    std::string result{};
    std::array<char, 128> buffer{};
    while (feof(pipe.get()) == 0) {
        if (fgets(buffer.data(), buffer.size(), pipe.get()) != nullptr) {
            result += buffer.data();
        }
    }

    if (result.ends_with('\n')) {
        result.pop_back();
    }

    return result;
}

std::string get_kernel_running() noexcept {
    return exec(R"(grep -Po '(?<=initrd\=\\initramfs-)(.+)(?=\.img)|(?<=boot\/vmlinuz-)([^ $]+)' /proc/cmdline)");
}

void kernel_repo(const AlpmHelper& alpm_helper, FILE* fd_obj = stdout) noexcept {
    const auto& kernels = alpm_helper.kernels();

    fmt::print(stderr, "\033[32mavailable kernels:\033[0m\n");
    for (const auto& kernel : kernels) {
        fmt::print(fd_obj, "{} {}\n", kernel.get_raw(), kernel.version());
    }
}

void kernel_list(const AlpmHelper& alpm_helper, FILE* fd_obj = stdout) noexcept {
    const auto& kernels = alpm_helper.kernels();

    const auto current_kernel = get_kernel_running();
    fmt::print(stderr, "\033[32mCurrently running:\033[0m {} ({})\n", exec("uname -r"), current_kernel);
    fmt::print(fd_obj, "The following kernels are installed in your system:\n");
    for (const auto& kernel : kernels) {
        /* clang-format off */
        if (!kernel.is_installed()) { continue; }
        /* clang-format on */
        fmt::print(fd_obj, "local/{} {}\n", kernel.name(), kernel.version());
    }
}

bool kernel_install(const AlpmHelper& alpm_helper, const std::vector<std::string>& kernels) noexcept {
    std::string pkginstall{};
    bool rmc{};

    const auto current_kernel = get_kernel_running();
    for (const auto& kernel : kernels) {
        if (kernel == "rmc") {
            rmc = true;
            continue;
        } else if (current_kernel == kernel) {
            err("You can't reinstall your current kernel. Please use 'pacman -Syu' instead to update.");
        } else if (ranges::none_of(alpm_helper.kernels(), [&](auto&& elem) {
                       return elem.name() == kernel;
                   })) {
            fmt::print(stderr, "\033[31mError:\033[0m Please make sure if the given kernel(s) exist(s).\n");
            kernel_repo(alpm_helper, stderr);
            return false;
        }

        pkginstall += fmt::format("{} ", kernel);
    }
    [[maybe_unused]] auto code = std::system("pacman -Syy");

    const auto& outofdate = exec("pacman -Qqu | tr '\n' ' '");
    if (!outofdate.empty()) {
        fmt::print(stderr, "The following packages are out of date, please update your system first: {}\n", outofdate);
        fmt::print("Do you want to continue anyway? [y/N] ");
        char yesno{'N'};
        std::cin >> yesno;
        fmt::print("\n");

        /* clang-format off */
        if (yesno != 'Y' && yesno != 'y') { return false; }
        /* clang-format on */
    }

    auto cmd              = fmt::format("pacman -Syu {}", pkginstall);
    const int status_code = std::system(cmd.c_str());

    cmd = fmt::format("pacman -R {}", current_kernel);
    if (rmc && (status_code == 0)) {
        code = std::system(cmd.c_str());
    } else if (rmc && (status_code != 0)) {
        err("\n'rmc' aborted because the kernel failed to install or canceled on removal.");
    }
    return true;
}

bool kernel_remove(const AlpmHelper& alpm_helper, const std::vector<std::string>& kernels) noexcept {
    std::string pkgremove{};

    const auto current_kernel = get_kernel_running();
    for (const auto& kernel : kernels) {
        if (kernel.empty()) {
            err("Invalid argument (use -h for help).");
        } else if (current_kernel == kernel) {
            err("You can't remove your current kernel.");
        } else if (!ranges::any_of(alpm_helper.kernels(), [&](auto&& elem) {
                       return elem.is_installed() && (elem.name() == kernel);
                   })) {
            fmt::print(stderr, "\033[31mError:\033[0m Kernel not installed.\n");
            kernel_list(alpm_helper, stderr);
            return false;
        }

        pkgremove += fmt::format("{} ", kernel);
    }

    const auto& cmd                   = fmt::format("pacman -R {}", pkgremove);
    [[maybe_unused]] const auto& code = std::system(cmd.c_str());
    return (code == 0);
}

}  // namespace

int main(int argc, char** argv) {
    const auto& process_args = [](auto&& args) {
        std::vector<std::string> kernels{};
        kernels.reserve(static_cast<size_t>(args.size() - 1));
        for (std::size_t i = 2; i < args.size(); ++i) {
            kernels.emplace_back(args[i]);
        }
        return kernels;
    };

    auto args = std::span{argv, static_cast<std::size_t>(argc)};
    if (args.size() < 2) {
        err("No arguments given (use -h for help).");
    }

    const std::string_view argument{args[1]};
    if (argument == "-h" || argument == "--help") {
        kernel_usage();
        return 0;
    } else if (argument == "-lr" || argument == "--listrunning") {
        const auto current_kernel = get_kernel_running();
        fmt::print("{}\n", current_kernel);
        return 0;
    }

    if (argument == "-l" || argument == "--list") {
        const auto alpm_helper = AlpmHelper();
        kernel_repo(alpm_helper);
        return 0;
    } else if (argument == "-li" || argument == "--listinstalled") {
        const auto alpm_helper = AlpmHelper();
        kernel_list(alpm_helper);
        return 0;
    }

    const bool is_install = (argument == "-i" || argument == "--install");
    const bool is_remove  = (argument == "-r" || argument == "--remove");
    const auto trans_func = (is_install) ? kernel_install : ((is_remove) ? kernel_remove : nullptr);

    if (is_install || is_remove) {
        root_check();

        const auto alpm_helper = AlpmHelper();
        const auto pos_args    = process_args(args);

        if (!trans_func(alpm_helper, pos_args)) {
            return 1;
        }
        return 0;
    }
    err("Invalid argument (use -h for help).");
}
