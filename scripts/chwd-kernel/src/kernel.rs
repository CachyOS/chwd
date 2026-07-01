// Copyright (C) 2022-2026 Vladislav Nepogodin
//
// This file is part of CachyOS chwd.
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

use std::cmp::Ordering;
use std::process::Command;

const IGNORED_PKG: &str = "linux-api-headers";
const HEADERS_SUFFIX: &str = "-headers";

/// A kernel package.
#[derive(Debug, Default, Clone)]
pub struct KernelPkg {
    pub name: String,
    pub repo: String,
    /// `repo/name`
    pub raw: String,
    /// Display version, prefixed with `∨` (older) or `∧` (update available).
    pub version: String,
    pub category: String,
    pub installed: bool,
    pub update_available: bool,
    /// Repo the package was installed from; empty when not installed.
    pub installed_db: String,
    pub headers_pkg: String,
    /// Module packages; empty when the sync db has no such package.
    pub zfs_pkg: String,
    pub nvidia_pkg: String,
    pub nvidia_open_pkg: String,
    pub is_aur: bool,
}

/// The packages a transaction should install/remove.
#[derive(Debug, Default, Clone)]
pub struct TransactionPlan {
    pub pacman_install: Vec<String>,
    pub pacman_remove: Vec<String>,
    pub aur_install: Vec<String>,
}

/// Open an alpm handle from the system pacman configuration.
pub fn open_alpm() -> alpm::Result<alpm::Alpm> {
    let pacman = pacmanconf::Config::with_opts(None, Some("/etc/pacman.conf"), Some("/")).unwrap();
    alpm_utils::alpm_with_conf(&pacman)
}

/// Human-readable category inferred from the kernel name.
pub fn category_of(name: &str) -> &'static str {
    const TABLE: &[(&str, &str)] = &[
        ("lto", "lto optimized"),
        ("lts", "longterm"),
        ("zen", "zen-kernel"),
        ("hardened", "hardened kernel"),
        ("deckify", "handheld kernel"),
        ("server", "server kernel"),
        ("next", "next release"),
        ("mainline", "mainline branch"),
        ("git", "master branch"),
        ("rc", "release candidate"),
    ];
    for (needle, label) in TABLE {
        if name.contains(needle) {
            return label;
        }
    }
    "stable"
}

/// Compare the installed version (if any) against the sync version, returning
/// the display string and whether an update is available.
fn decorated_version(handle: &alpm::Alpm, name: &str, sync_ver: &str) -> (String, bool) {
    let Ok(local_pkg) = handle.localdb().pkg(name.as_bytes()) else {
        return (sync_ver.to_owned(), false);
    };
    let local_ver = local_pkg.version().as_str().to_owned();
    match alpm::vercmp(local_ver.as_str(), sync_ver) {
        Ordering::Greater => (format!("\u{2228}{local_ver}"), false),
        Ordering::Less => (format!("\u{2227}{sync_ver}"), true),
        Ordering::Equal => (sync_ver.to_owned(), false),
    }
}

#[inline]
fn syncdb_has(db: &alpm::Db, name: &str) -> bool {
    db.pkg(name.as_bytes()).is_ok()
}

#[inline]
fn localdb_has(handle: &alpm::Alpm, name: &str) -> bool {
    handle.localdb().pkg(name.as_bytes()).is_ok()
}

mod alpm_ffi {
    use std::os::raw::{c_char, c_void};

    unsafe extern "C" {
        pub fn alpm_db_get_pkg(db: *mut c_void, name: *const c_char) -> *mut c_void;
        pub fn alpm_pkg_get_installed_db(pkg: *mut c_void) -> *const c_char;
    }
}

/// Repo a package was InstalledFrom.
fn installed_db_name(local_db: &alpm::Db, name: &str) -> String {
    let Ok(cname) = std::ffi::CString::new(name) else {
        return String::new();
    };
    // SAFETY: localdb owned by the alpm handle. Both calls
    // return borrowed pointers owned by libalpm, and the
    // returned C string outlives this call.
    unsafe {
        let pkg = alpm_ffi::alpm_db_get_pkg(local_db.as_ptr().cast(), cname.as_ptr());
        if pkg.is_null() {
            return String::new();
        }
        let db_name = alpm_ffi::alpm_pkg_get_installed_db(pkg);
        if db_name.is_null() {
            return String::new();
        }
        std::ffi::CStr::from_ptr(db_name).to_string_lossy().into_owned()
    }
}

/// alpm version comparison exposed.
pub fn vercmp(a: &str, b: &str) -> i32 {
    match alpm::vercmp(a, b) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

/// Fill in the zfs/nvidia module packages that actually exist for kern.
fn attach_modules(kernel: &mut KernelPkg, db: &alpm::Db) {
    // keep the name only if the sync db actually has that package
    let module = |pkgname: String| if syncdb_has(db, &pkgname) { pkgname } else { String::new() };

    let name = &kernel.name;
    if name.starts_with("linux-cachyos") {
        kernel.zfs_pkg = module(format!("{name}-zfs"));
        kernel.nvidia_pkg = module(format!("{name}-nvidia"));
        kernel.nvidia_open_pkg = module(format!("{name}-nvidia-open"));
    } else if name == "linux" || name == "linux-lts" {
        let suffix = &name["linux".len()..];
        kernel.nvidia_pkg = module(format!("nvidia{suffix}"));
        kernel.nvidia_open_pkg = module(format!("nvidia-open{suffix}"));
    }
}

/// Discover every kernel across the sync databases.
pub fn get_kernels(handle: &alpm::Alpm) -> Vec<KernelPkg> {
    let needles: &[String] = &["linux[^ ]*-headers".into()];
    let mut kernels = Vec::new();

    for db in handle.syncdbs() {
        let db_name = db.name();
        let Ok(found) = db.search(needles.iter()) else {
            continue;
        };

        for header_pkg in found.iter() {
            let header_name = header_pkg.name();
            if header_name.contains(IGNORED_PKG) {
                continue;
            }
            let base_name = header_name.strip_suffix(HEADERS_SUFFIX).unwrap_or(header_name);

            let Ok(base_pkg) = db.pkg(base_name.as_bytes()) else {
                continue;
            };
            let sync_ver = base_pkg.version().as_str().to_owned();
            let (version, update_available) = decorated_version(handle, base_name, &sync_ver);

            let mut kernel = KernelPkg {
                name: base_name.to_owned(),
                repo: db_name.to_owned(),
                raw: format!("{db_name}/{base_name}"),
                version,
                category: category_of(base_name).to_owned(),
                installed: localdb_has(handle, base_name),
                update_available,
                installed_db: installed_db_name(handle.localdb(), base_name),
                headers_pkg: header_name.to_owned(),
                ..Default::default()
            };
            attach_modules(&mut kernel, db);
            kernels.push(kernel);
        }
    }

    #[cfg(feature = "aur")]
    append_aur_kernels(&mut kernels);

    kernels
}

/// The prebuilt nvidia that chwd reports as an installed profile.
struct NvidiaPrebuilt {
    dkms: bool,
    open: bool,
}

pub fn shell_capture(cmd: &str) -> String {
    Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .unwrap_or_default()
}

fn is_root_on_zfs() -> bool {
    shell_capture("findmnt -ln -o FSTYPE /") == "zfs"
}

/// Ask chwd which nvidia driver profile is installed.
fn nvidia_prebuilt() -> NvidiaPrebuilt {
    let profiles =
        shell_capture("chwd --list-installed -d 2>/dev/null | grep Name | awk '{print $4}'");
    let mut result = NvidiaPrebuilt { dkms: false, open: false };
    for profile in profiles.lines() {
        if profile.starts_with("nvidia-open-dkms") {
            result.open = true;
        } else if profile.starts_with("nvidia-dkms") {
            result.dkms = true;
        }
    }
    result
}

/// Whether an installed `linux-cachyos*` nvidia module of each kern exists,
/// in a single scan.
fn installed_cachyos_nvidia_modules(handle: &alpm::Alpm) -> (bool, bool) {
    let (mut has_nvidia, mut has_open) = (false, false);
    for pkg in handle.localdb().pkgs() {
        let name = pkg.name();
        if !name.starts_with("linux-cachyos") {
            continue;
        }
        if name.ends_with("-nvidia-open") {
            has_open = true;
        } else if name.ends_with("-nvidia") {
            has_nvidia = true;
        }
    }
    (has_nvidia, has_open)
}

/// Queue packages to install for one kernel.
fn resolve_install_one(kernel: &KernelPkg, ctx: &InstallContext, plan: &mut TransactionPlan) {
    if kernel.is_aur {
        plan.aur_install.push(kernel.name.clone());
        return;
    }

    if ctx.root_on_zfs && !kernel.zfs_pkg.is_empty() {
        plan.pacman_install.push(kernel.zfs_pkg.clone());
    }

    let dkms_modules_not_installed = !ctx.nvidia_dkms_installed && !ctx.nvidia_open_dkms_installed;

    // Prefer already installed fallback to the chwd-detected prebuilt.
    let (should_install_nvidia, should_install_nvidia_open) =
        if ctx.nvidia_open_modules_installed && !kernel.nvidia_open_pkg.is_empty() {
            (false, true)
        } else if ctx.nvidia_modules_installed && !kernel.nvidia_pkg.is_empty() {
            (true, false)
        } else {
            (
                ctx.prebuilt.dkms && !kernel.nvidia_pkg.is_empty(),
                ctx.prebuilt.open && !kernel.nvidia_open_pkg.is_empty(),
            )
        };

    if dkms_modules_not_installed && should_install_nvidia_open {
        plan.pacman_install.push(kernel.nvidia_open_pkg.clone());
    } else if dkms_modules_not_installed && should_install_nvidia {
        plan.pacman_install.push(kernel.nvidia_pkg.clone());
    }

    plan.pacman_install.push(kernel.name.clone());
    plan.pacman_install.push(kernel.headers_pkg.clone());
}

/// Queue packages to remove for one kernel.
fn resolve_remove_one(handle: &alpm::Alpm, kernel: &KernelPkg, plan: &mut TransactionPlan) {
    if !kernel.installed {
        return;
    }
    plan.pacman_remove.push(kernel.name.clone());

    for module in
        [&kernel.headers_pkg, &kernel.zfs_pkg, &kernel.nvidia_pkg, &kernel.nvidia_open_pkg]
    {
        if !module.is_empty() && localdb_has(handle, module) {
            plan.pacman_remove.push(module.clone());
        }
    }
}

/// State per transaction.
struct InstallContext {
    root_on_zfs: bool,
    prebuilt: NvidiaPrebuilt,
    nvidia_dkms_installed: bool,
    nvidia_open_dkms_installed: bool,
    nvidia_modules_installed: bool,
    nvidia_open_modules_installed: bool,
}

impl InstallContext {
    fn probe(handle: &alpm::Alpm) -> Self {
        let (nvidia_modules_installed, nvidia_open_modules_installed) =
            installed_cachyos_nvidia_modules(handle);
        Self {
            root_on_zfs: is_root_on_zfs(),
            prebuilt: nvidia_prebuilt(),
            nvidia_dkms_installed: localdb_has(handle, "nvidia-dkms"),
            nvidia_open_dkms_installed: localdb_has(handle, "nvidia-open-dkms"),
            nvidia_modules_installed,
            nvidia_open_modules_installed,
        }
    }
}

/// Build the full transaction plan for the given install/remove selections.
pub fn resolve_transaction(
    handle: &alpm::Alpm,
    kernels: &[KernelPkg],
    install_raws: &[String],
    remove_raws: &[String],
) -> TransactionPlan {
    let mut plan = TransactionPlan::default();
    let find = |raw: &str| kernels.iter().find(|k| k.raw == raw);

    if !install_raws.is_empty() {
        let ctx = InstallContext::probe(handle);
        for raw in install_raws {
            if let Some(kernel) = find(raw) {
                resolve_install_one(kernel, &ctx, &mut plan);
            }
        }
    }

    for raw in remove_raws {
        if let Some(kernel) = find(raw) {
            resolve_remove_one(handle, kernel, &mut plan);
        }
    }

    plan
}

/// Extract the running kernel name from `/proc/cmdline`.
pub fn running_kernel() -> String {
    shell_capture(
        r"grep -Po '(?<=initrd\=\\initramfs-)(.+)(?=\.img)|(?<=boot\/vmlinuz-)([^ $]+)' /proc/cmdline",
    )
}

/// Append AUR kernels not already provided by the official repos. Requires
/// `paru` and `awk`, and only runs once official kernels have been found.
#[cfg(feature = "aur")]
fn append_aur_kernels(kernels: &mut Vec<KernelPkg>) {
    use std::path::Path;

    if kernels.is_empty() {
        return;
    }
    if !Path::new("/sbin/paru").exists() {
        eprintln!("Paru is not installed! Disabling AUR kernels support");
        return;
    }

    let headers = shell_capture("paru --aur -Sl | grep ' linux[^ ]*-headers' | awk '{print $2}'");
    for header in headers.lines() {
        let name = header.replace("-headers", "");
        if kernels.iter().any(|k| k.name == name) {
            continue;
        }
        kernels.push(KernelPkg {
            name: name.clone(),
            repo: "aur".to_owned(),
            raw: format!("aur/{name}"),
            version: "unknown-version".to_owned(),
            category: category_of(&name).to_owned(),
            headers_pkg: header.to_owned(),
            is_aur: true,
            ..Default::default()
        });
    }
}

/// Clone or refresh an AUR package.
#[cfg(feature = "aur")]
fn prepare_git_repo(parent_dir: &std::path::Path, repo_path: &std::path::Path, clone_url: &str) {
    use std::process::Command;

    let _ = std::fs::create_dir_all(parent_dir);

    // A leftover non-git directory would make `git clone` fail; drop it.
    if repo_path.exists() && !repo_path.join(".git").exists() {
        let _ = std::fs::remove_dir_all(repo_path);
    }

    let git = |dir: &std::path::Path, args: &[&str]| {
        Command::new("git")
            .current_dir(dir)
            .args(args)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };

    if !repo_path.exists() {
        let basename = repo_path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        if !git(parent_dir, &["clone", clone_url, basename]) {
            eprintln!("prepare_git_repo: 'git clone {clone_url}' failed");
            return;
        }
    }

    if !git(repo_path, &["checkout", "--force", "master"])
        || !git(repo_path, &["clean", "-fd"])
        || !git(repo_path, &["pull"])
    {
        eprintln!("prepare_git_repo: failed to refresh checkout at '{}'", repo_path.display());
    }
}

/// Clone and `makepkg` each of the given AUR kernels.
#[cfg(feature = "aur")]
pub fn install_aur_kernels(kernel_names: &[String]) {
    use std::path::PathBuf;
    use std::process::Command;

    let home = std::env::var("HOME").unwrap_or_default();
    let pkgbuilds_dir = PathBuf::from(&home).join(".cache/cachyos-km/aur_pkgbuilds");

    for kernel_name in kernel_names {
        if kernel_name.contains("headers") {
            continue;
        }

        let package_path = pkgbuilds_dir.join(kernel_name);
        prepare_git_repo(
            &pkgbuilds_dir,
            &package_path,
            &format!("https://aur.archlinux.org/{kernel_name}.git"),
        );

        let _ = Command::new("makepkg")
            .current_dir(&package_path)
            .args(["-sicf", "--cleanbuild", "--skipchecksums"])
            .status();
    }
}

/// No-op stand-in so callers don't need their own `cfg`s when AUR is disabled.
#[cfg(not(feature = "aur"))]
pub fn install_aur_kernels(_kernel_names: &[String]) {
    eprintln!("AUR kernel installation is not supported in this build");
}
