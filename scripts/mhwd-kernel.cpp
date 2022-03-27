// Copyright (C) 2022 Vladislav Nepogodin
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

namespace {
class AlpmHelper {
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

void err(const std::string_view& msg) noexcept {
    fmt::print(stderr, "\033[31mError:\033[0m {}\n", msg);
    std::exit(1);
}

void root_check() noexcept {
    if (geteuid() == 0) return;
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
    if (!pipe) {
        return "-1";
    }

    std::string result{};
    std::array<char, 128> buffer{};
    while (!feof(pipe.get())) {
        if (fgets(buffer.data(), buffer.size(), pipe.get()) != nullptr) {
            result += buffer.data();
        }
    }

    if (result.ends_with('\n')) {
        result.pop_back();
    }

    return result;
}

std::string kernel_running() noexcept {
    return exec(R"(grep -Po '(?<=initrd\=\\initramfs-)(.+)(?=\.img)|(?<=boot\/vmlinuz-)([^ $]+)' /proc/cmdline)");
}
const auto current = kernel_running();
const auto helper  = AlpmHelper();

void kernel_repo(FILE* fd = stdout) noexcept {
    const auto& kernels = helper.kernels();

    fmt::print(stderr, "\033[32mavailable kernels:\033[0m\n");
    for (const auto& kernel : kernels) {
        fmt::print(fd, "{} {}\n", kernel.get_raw(), kernel.version());
    }
}

void kernel_list(FILE* fd = stdout) noexcept {
    const auto& kernels = helper.kernels();

    fmt::print(stderr, "\033[32mCurrently running:\033[0m {} ({})\n", exec("uname -r"), current);
    fmt::print(fd, "The following kernels are installed in your system:\n");
    for (const auto& kernel : kernels) {
        if (!kernel.is_installed()) { continue; }
        fmt::print(fd, "local/{} {}\n", kernel.name(), kernel.version());
    }
}

void kernel_install(const std::vector<std::string>& kernels) noexcept {
    std::string pkginstall{};
    bool rmc{};

    for (const auto& kernel : kernels) {
        if (kernel == "rmc") { rmc = true; continue; }
        else if (current == kernel) {
            err("You can't reinstall your current kernel. Please use 'pacman -Syu' instead to update.");
        } else if (ranges::none_of(helper.kernels(), [&](auto&& elem) {
                       return elem.name() == kernel;
                   })) {
            fmt::print(stderr, "\033[31mError:\033[0m Please make sure if the given kernel(s) exist(s).\n");
            kernel_repo(stderr);
            std::exit(1);
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
        if (yesno != 'Y' && yesno != 'y') { std::exit(1); }
    }

    auto cmd              = fmt::format("pacman -Syu \"{}\"", pkginstall);
    const int status_code = std::system(cmd.c_str());

    cmd = fmt::format("pacman -R \"{}\"", current);
    if (rmc && (status_code == 0)) {
        code = std::system(cmd.c_str());
    } else if (rmc && (status_code != 0)) {
        err("\n'rmc' aborted because the kernel failed to install or canceled on removal.");
    }
}

void kernel_remove(const std::vector<std::string>& kernels) noexcept {
    std::string pkgremove{};

    for (const auto& kernel : kernels) {
        if (kernel.empty()) { err("Invalid argument (use -h for help)."); }
        else if (current == kernel) { err("You can't remove your current kernel."); }
        else if (!ranges::any_of(helper.kernels(), [&](auto&& elem) {
                       return elem.is_installed() && (elem.name() == kernel);
                   })) {
            fmt::print(stderr, "\033[31mError:\033[0m Kernel not installed.\n");
            kernel_list(stderr);
            std::exit(1);
        }

        pkgremove += fmt::format("{} ", kernel);
    }

    const auto& cmd = fmt::format("pacman -R \"{}\"", pkgremove);
    [[maybe_unused]] const auto& code = std::system(cmd.c_str());
}

}  // namespace

int main(int argc, char** argv) {
    if (argc < 2) {
        err("No arguments given (use -h for help).");
    }

    const auto& process_args = [argc, argv]() {
        std::vector<std::string> kernels{};
        kernels.reserve(static_cast<size_t>(argc - 1));
        for (int i = 2; i < argc; ++i) {
            kernels.emplace_back(argv[i]);
        }
        return kernels;
    };
    const std::string_view argument = argv[1];
    if (argument == "-h" || argument == "--help") {
        kernel_usage();
        return 0;
    } else if (argument == "-i" || argument == "--install") {
        root_check();
        kernel_install(process_args());
        return 0;
    } else if (argument == "-l" || argument == "--list") {
        kernel_repo();
        return 0;
    } else if (argument == "-li" || argument == "--listinstalled") {
        kernel_list();
        return 0;
    } else if (argument == "-lr" || argument == "--listrunning") {
        fmt::print("{}\n", current);
        return 0;
    } else if (argument == "-r" || argument == "--remove") {
        root_check();
        kernel_remove(process_args());
        return 0;
    }
    err("Invalid argument (use -h for help).");
}
