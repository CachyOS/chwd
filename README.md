<div align="center">
  <h1>chwd</h1>
  <p>
    <strong>CachyOS Hardware Detection Tool written in Rust</strong>
  </p>
  <p>

[![Dependency Status](https://deps.rs/repo/github/cachyos/chwd/status.svg)](https://deps.rs/repo/github/cachyos/chwd)
<br />
[![CI](https://github.com/cachyos/chwd/actions/workflows/rust.yml/badge.svg)](https://github.com/cachyos/chwd/actions/workflows/rust.yml)

  </p>
</div>

`chwd` (CachyOS Hardware Detection) is a powerful, Rust-based command-line utility designed to take the headache out of hardware configuration on Linux. It automatically detects your system's hardware components (like GPUs, Network Interface Cards) and applies the optimal, verified driver profiles so your hardware "just works."

Built primarily for CachyOS `chwd` replaces manual installation drivers through `pacman` and manual configuration.

## Key Features

- **Automatic Configuration**: Detects connected hardware and automatically installs correct driver profiles.
- **Targeted Installation**: Allows installing or removing specific hardware profiles easily (e.g., reinstalling drivers when swapping GPUs).
- **Comprehensive Listings**: Checks what profiles are currently installed, which profiles your current hardware supports, or all known profiles in the database.
- **Safety Checks**: Checks using `libpci` and verifies hardware presence before applying configurations to prevent system breakage.

## Quick Start

### Installation

`chwd` comes **pre-installed on CachyOS**, so no action is required out of the box. If you are using **Arch Linux**, you can install `chwd` directly from the [CachyOS repositories](https://github.com/CachyOS/linux-cachyos?tab=readme-ov-file#cachyos-repositories):

```bash
sudo pacman -S chwd
```

### Basic Usage

For detailed usage instructions, please refer to the [chwd documentation on the CachyOS Wiki](https://wiki.cachyos.org/features/chwd/chwd/).

## Under the Hood (For Contributors)

`chwd` uses `.toml` files (can be found under `profiles/`) to manage hardware profiles. 

When you run an `--install` or `-a` command, `chwd` checks the PCI devices on your machine against the available TOML profiles.

## Building from Source

If you want to contribute, test the latest untested changes, or just build it yourself:

1.  **Install build dependencies:**
    ```bash
    sudo pacman -S --needed rust pciutils
    ```
2.  **Clone the repository:**
    ```bash
    git clone https://github.com/cachyos/chwd.git
    cd chwd
    ```
3.  **Build the project:**
    ```bash
    cargo build --release
    ```
    The compiled binary will be located in `./target/release/chwd`.

## License

This project is licensed under the **GNU General Public License v3.0**. See the [LICENSE](LICENSE) file for full details.
