<div align="center">
  <h1>chwd</h1>
  <p>
    <strong>CachyOS Hardware Detection Tool written in Rust</strong>
  </p>
  <p>

[![Dependency Status](https://deps.rs/repo/github/cachyos/chwd/status.svg)](https://deps.rs/repo/github/cachyos/chwd)
[![CI](https://github.com/cachyos/chwd/actions/workflows/rust.yml/badge.svg)](https://github.com/cachyos/chwd/actions/workflows/rust.yml)

  </p>
</div>

`chwd` (CachyOS Hardware Detection) is a powerful, Rust-based command-line utility designed to take the headache out of hardware configuration on Linux. It automatically detects your system's hardware components (like GPUs, Network Interface Cards) and applies the optimal, verified driver profiles so your hardware "just works."

Built primarily for CachyOS and Arch Linux, `chwd` replaces tedious manual `pacman` driver installations and configuration file creations with single-command solutions.

## Key Features

*   **Automatic Configuration**: Detects connected hardware and automatically installs the most appropriate driver profiles in one command.
*   **Targeted Installation**: Allows installing or removing specific hardware profiles easily (e.g., swapping between open-source and proprietary graphics drivers).
*   **Profile Listings**: Quickly see what profiles are currently installed, which profiles your current hardware supports, or all known profiles in the database.
*   **AI SDK Support**: Includes a toggle to install specific hardware profiles tailored for AI and Machine Learning developers.
*   **Safety Checks**: Integrates deep checks using `libpci` and verifies hardware presence before applying configurations to prevent system breakage.

## Quick Start

### Prerequisites
*   An Arch-based Linux distribution (CachyOS, Arch Linux).
*   `pacman` package manager installed and configured.
*   `sudo` privileges (modifying drivers requires root access).

### Installation (Tool)
On CachyOS, you can install `chwd` directly from the repositories:
```bash
sudo pacman -S chwd
```

### Basic Usage

**1. See what hardware profiles you can install right now:**
```bash
chwd --list
```

**2. Make it all work automatically (Autoconfigure):**
```bash
sudo chwd -a
```

**3. Install a specific profile explicitly (e.g., if you know you need `video-nvidia`):**
```bash
sudo chwd -i video-nvidia
```

**4. See what hardware profiles are currently installed on your system:**
```bash
chwd --list-installed
```

## Detailed CLI Options

If you prefer seeing the options directly from the terminal, running `chwd --help` provides standard usage formatting:

```bash
❯ chwd --help
Usage: chwd [OPTIONS]

Options:
  -c, --check <profile>            Check profile
  -i, --install <profile>          Install profile
  -r, --remove <profile>           Remove profile
  -d, --detail                     Show detailed info for listings
  -f, --force                      Force reinstall
      --list-installed             List installed kernels
      --list                       List available profiles for all devices
      --list-all                   List all profiles
  -a, --autoconfigure [<classid>]  Autoconfigure
      --ai_sdk                     Toggle AI SDK profiles
      --pmcachedir <PMCACHEDIR>    [default: /var/cache/pacman/pkg]
      --pmconfig <PMCONFIG>        [default: /etc/pacman.conf]
      --pmroot <PMROOT>            [default: /]
  -h, --help                       Print help
  -V, --version                    Print version
```

## Under the Hood (For Contributors)

`chwd` uses `.toml` files (typically found under `profiles/pci/`) to manage hardware profiles. 

When you run an `--install` or `-a` command, `chwd` checks the PCI devices on your machine against these TOML profiles by matching `class_ids`, `vendor_ids`, and `device_ids`. 

If a match is found, the profile dictates what `packages` Pacman should pull (e.g., `nvidia-utils`, `vulkan-icd-loader`), and executes any defined bash instructions in the `pre_install` or `post_install` fields seamlessly.

## Building from Source

If you want to contribute, test the latest untested changes, or just build it yourself:

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/cachyos/chwd.git
    cd chwd
    ```
2.  **Ensure you have Rust and Cargo installed:**
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
3.  **Build the project:**
    Requirements like `libpci` need to be present on your system.
    ```bash
    cargo build --release
    ```
    The compiled binary will be located in `./target/release/chwd`.

## License

This project is licensed under the **GNU General Public License v3.0**. See the [LICENSE](LICENSE) file for full details.
