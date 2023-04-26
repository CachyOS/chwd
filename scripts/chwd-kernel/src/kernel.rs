// Copyright (C) 2022-2023 Vladislav Nepogodin
//
// This file is part of CachyOS kernel manager.
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

const IGNORED_PKG: &str = "linux-api-headers";
const REPLACE_PART: &str = "-headers";
// const NEEDLE: &str = "linux[^ ]*-headers";

#[derive(Debug)]
pub struct Kernel<'a> {
    pub name: String,
    pub repo: String,
    pub raw: String,
    alpm_pkg: Option<alpm::Package<'a>>,
    alpm_handle: Option<&'a alpm::Alpm>,
}

impl Kernel<'_> {
    // Name must be without any repo name (e.g. core/linux)
    pub fn is_installed(&self) -> Option<bool> {
        let local_db = self.alpm_handle.as_ref()?.localdb();
        Some(local_db.pkg(self.name.as_bytes()).is_ok())
    }

    pub fn version(&self) -> Option<String> {
        if !self.is_installed().unwrap() {
            return Some(self.alpm_pkg.as_ref()?.version().to_string());
        }

        let local_db = self.alpm_handle.as_ref()?.localdb();
        let local_pkg = local_db.pkg(self.name.as_bytes());

        Some(local_pkg.ok()?.version().to_string())
    }
}

/// Find kernel packages by finding packages which have words 'linux' and 'headers'.
/// From the output of 'pacman -Sl'
/// - find lines that have words: 'linux' and 'headers'
/// - drop lines containing 'testing' (=testing repo, causes duplicates) and 'linux-api-headers'
///   (=not a kernel header)
/// - show the (header) package names
/// Now we have names of the kernel headers.
/// Then add the kernel packages to proper places and output the result.
/// Then display possible kernels and headers added by the user.

/// The output consists of a list of reponame and a package name formatted as: "reponame/pkgname"
/// For example:
///    reponame/linux-xxx reponame/linux-xxx-headers
///    reponame/linux-yyy reponame/linux-yyy-headers
///    ...
pub fn get_kernels(alpm_handle: &alpm::Alpm) -> Vec<Kernel> {
    let mut kernels = Vec::new();
    let needles: &[String] = &["linux-[a-z]".into(), "headers".into()];
    // let needles: &[String] = &["linux[^ ]*-headers".into()];

    for db in alpm_handle.syncdbs() {
        let db_name = db.name();
        // search each database for packages matching the regex "linux-[a-z]" AND "headers"
        for pkg_headers in db.search(needles.iter()).unwrap() {
            let mut pkg_name = pkg_headers.name().to_owned();
            if pkg_name.contains(IGNORED_PKG) {
                continue;
            }
            pkg_name = pkg_name.replace(REPLACE_PART, "");

            // Skip if the actual kernel package is not found
            if let Ok(pkg) = db.pkg(pkg_name.as_bytes()) {
                kernels.push(Kernel {
                    name: pkg_name.clone(),
                    repo: db_name.to_string(),
                    raw: format!("{}/{}", db_name, pkg_name),

                    alpm_pkg: Some(pkg),
                    alpm_handle: Some(alpm_handle),
                });
            }
        }
    }

    kernels
}
