// Copyright (C) 2023-2024 Vladislav Nepogodin
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

use crate::fl;
use anyhow::Result;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use std::fs;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct HardwareID {
    pub class_ids: Vec<String>,
    pub vendor_ids: Vec<String>,
    pub device_ids: Vec<String>,
    pub blacklisted_class_ids: Vec<String>,
    pub blacklisted_vendor_ids: Vec<String>,
    pub blacklisted_device_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub is_nonfree: bool,

    pub is_ai_sdk: bool,

    pub prof_path: String,
    pub name: String,
    pub desc: String,
    pub priority: i32,
    pub packages: String,
    pub post_install: String,
    pub post_remove: String,
    pub device_name_pattern: Option<String>,
    pub hwd_product_name_pattern: Option<String>,
    pub gc_versions: Option<Vec<String>>,

    pub hwd_ids: Vec<HardwareID>,
}

impl Default for Profile {
    fn default() -> Self {
        Self::new()
    }
}

impl Profile {
    pub fn new() -> Self {
        Self { hwd_ids: vec![Default::default()], ..Default::default() }
    }
}

pub fn parse_profiles(file_path: &str) -> Result<Vec<Profile>> {
    let mut profiles = vec![];
    let file_content = fs::read_to_string(file_path)?;
    let toml_table = file_content.parse::<toml::Table>()?;

    for (key, value) in toml_table.iter() {
        if !value.is_table() {
            continue;
        }
        let value_table = value.as_table().unwrap();

        let toplevel_profile = parse_profile(value_table, key);
        if toplevel_profile.is_err() {
            continue;
        }

        for (nested_key, nested_value) in value_table.iter() {
            if !nested_value.is_table() {
                continue;
            }
            let nested_profile_name = format!("{}.{}", key, nested_key);
            let mut nested_value_table = nested_value.as_table().unwrap().clone();
            merge_table_left(&mut nested_value_table, value_table);
            let nested_profile = parse_profile(&nested_value_table, &nested_profile_name);
            if nested_profile.is_err() {
                continue;
            }
            let mut nested_profile = nested_profile?;
            file_path.clone_into(&mut nested_profile.prof_path);
            profiles.push(nested_profile);
        }
        let mut toplevel_profile = toplevel_profile?;
        file_path.clone_into(&mut toplevel_profile.prof_path);
        profiles.push(toplevel_profile);
    }

    Ok(profiles)
}

pub fn get_invalid_profiles(file_path: &str) -> Result<Vec<String>> {
    let mut invalid_profile_list = vec![];
    let file_content = fs::read_to_string(file_path)?;
    let toml_table = file_content.parse::<toml::Table>()?;

    for (key, value) in toml_table.iter() {
        if !value.is_table() {
            continue;
        }
        let value_table = value.as_table().unwrap();

        let toplevel_profile = parse_profile(value_table, key);
        if toplevel_profile.is_err() {
            invalid_profile_list.push(key.to_owned());
            continue;
        }

        for (nested_key, nested_value) in value_table.iter() {
            if !nested_value.is_table() {
                continue;
            }
            let nested_profile_name = format!("{}.{}", key, nested_key);
            let mut nested_value_table = nested_value.as_table().unwrap().clone();
            merge_table_left(&mut nested_value_table, value_table);
            let nested_profile = parse_profile(&nested_value_table, &nested_profile_name);
            if nested_profile.is_ok() {
                continue;
            }
            invalid_profile_list.push(nested_profile_name);
        }
    }

    Ok(invalid_profile_list)
}

fn parse_profile(node: &toml::Table, profile_name: &str) -> Result<Profile> {
    let mut profile = Profile {
        is_nonfree: node.get("nonfree").and_then(|x| x.as_bool()).unwrap_or(false),
        is_ai_sdk: node.get("ai_sdk").and_then(|x| x.as_bool()).unwrap_or(false),
        prof_path: "".to_owned(),
        name: profile_name.to_owned(),
        packages: node.get("packages").and_then(|x| x.as_str()).unwrap_or("").to_owned(),
        post_install: node.get("post_install").and_then(|x| x.as_str()).unwrap_or("").to_owned(),
        post_remove: node.get("post_remove").and_then(|x| x.as_str()).unwrap_or("").to_owned(),
        desc: node.get("desc").and_then(|x| x.as_str()).unwrap_or("").to_owned(),
        priority: node.get("priority").and_then(|x| x.as_integer()).unwrap_or(0) as i32,
        hwd_ids: vec![Default::default()],
        device_name_pattern: node
            .get("device_name_pattern")
            .and_then(|x| x.as_str().map(str::to_string)),
        hwd_product_name_pattern: node
            .get("hwd_product_name_pattern")
            .and_then(|x| x.as_str().map(str::to_string)),
        gc_versions: node.get("gc_versions").and_then(|x| {
            x.as_str()
                .map(str::split_ascii_whitespace)
                .map(|x| x.map(str::to_string).collect::<Vec<_>>())
        }),
    };

    let conf_devids = node.get("device_ids").and_then(|x| x.as_str()).unwrap_or("");
    let conf_vendorids = node.get("vendor_ids").and_then(|x| x.as_str()).unwrap_or("");
    let conf_classids = node.get("class_ids").and_then(|x| x.as_str()).unwrap_or("");

    // Read ids in extern file
    let devids_val = if !conf_devids.is_empty() && conf_devids.as_bytes()[0] == b'>' {
        parse_ids_file(&conf_devids[1..])?
    } else {
        conf_devids.to_owned()
    };

    // Add new HardwareIDs group to vector if vector is not empty
    if !profile.hwd_ids.last().unwrap().device_ids.is_empty() {
        profile.hwd_ids.push(Default::default());
    }
    profile.hwd_ids.last_mut().unwrap().device_ids =
        devids_val.split(' ').filter(|x| !x.is_empty()).map(|x| x.to_owned()).collect::<Vec<_>>();
    if !profile.hwd_ids.last().unwrap().class_ids.is_empty() {
        profile.hwd_ids.push(Default::default());
    }
    profile.hwd_ids.last_mut().unwrap().class_ids = conf_classids
        .split(' ')
        .filter(|x| !x.is_empty())
        .map(|x| x.to_owned())
        .collect::<Vec<_>>();

    if !conf_vendorids.is_empty() {
        // Add new HardwareIDs group to vector if vector is not empty
        if !profile.hwd_ids.last().unwrap().vendor_ids.is_empty() {
            profile.hwd_ids.push(Default::default());
        }
        profile.hwd_ids.last_mut().unwrap().vendor_ids = conf_vendorids
            .split(' ')
            .filter(|x| !x.is_empty())
            .map(|x| x.to_owned())
            .collect::<Vec<_>>();
    }

    let append_star = |vec: &mut Vec<_>| {
        if vec.is_empty() {
            vec.push("*".to_string());
        }
    };

    // Append * to all empty vectors
    for hwd_id in profile.hwd_ids.iter_mut() {
        append_star(&mut hwd_id.class_ids);
        append_star(&mut hwd_id.vendor_ids);
        append_star(&mut hwd_id.device_ids);
    }
    Ok(profile)
}

fn parse_ids_file(file_path: &str) -> Result<String> {
    use std::fmt::Write;

    let file_content = fs::read_to_string(file_path)?;
    let parsed_ids = file_content
        .lines()
        .filter(|x| !x.trim().is_empty() && x.trim().as_bytes()[0] != b'#')
        .fold(String::new(), |mut output, x| {
            let _ = write!(output, " {}", x.trim());
            output
        });

    Ok(parsed_ids.split_ascii_whitespace().collect::<Vec<_>>().join(" "))
}

fn merge_table_left(lhs: &mut toml::Table, rhs: &toml::Table) {
    for (rhs_key, rhs_val) in rhs {
        // rhs key not found in lhs - direct move
        if lhs.get(rhs_key).is_none() {
            lhs.insert(rhs_key.to_string(), rhs_val.clone());
        }
    }
}

pub fn write_profile_to_file(file_path: &str, profile: &Profile) -> bool {
    let mut table = toml::Table::new();
    table.insert("nonfree".to_owned(), profile.is_nonfree.into());
    table.insert("ai_sdk".to_owned(), profile.is_ai_sdk.into());
    table.insert("desc".to_owned(), profile.desc.clone().into());
    table.insert("packages".to_owned(), profile.packages.clone().into());
    table.insert("priority".to_owned(), profile.priority.into());

    if !profile.post_install.is_empty() {
        table.insert("post_install".to_owned(), profile.post_install.clone().into());
    }
    if !profile.post_remove.is_empty() {
        table.insert("post_remove".to_owned(), profile.post_remove.clone().into());
    }
    if let Some(dev_name_pattern) = &profile.device_name_pattern {
        table.insert("device_name_pattern".to_owned(), dev_name_pattern.clone().into());
    }
    if let Some(product_name_pattern) = &profile.hwd_product_name_pattern {
        table.insert("hwd_product_name_pattern".to_owned(), product_name_pattern.clone().into());
    }
    if let Some(gc_versions) = &profile.gc_versions {
        table.insert("gc_versions".to_owned(), gc_versions.clone().into());
    }

    let last_hwd_id = profile.hwd_ids.last().unwrap();

    let device_ids = &last_hwd_id.device_ids;
    let vendor_ids = &last_hwd_id.vendor_ids;
    let class_ids = &last_hwd_id.class_ids;
    table.insert("device_ids".to_owned(), device_ids.join(" ").into());
    table.insert("vendor_ids".to_owned(), vendor_ids.join(" ").into());
    table.insert("class_ids".to_owned(), class_ids.join(" ").into());

    let toml_string = format!(
        "[{}]\n{}",
        profile.name,
        // NOTE: workaround toml escaping multiline raw strings
        toml::to_string_pretty(&table).unwrap().replace("\\\"", "\"")
    );
    fs::write(file_path, toml_string).is_ok()
}

pub fn print_profile_details(profile: &Profile) {
    let mut class_ids = String::new();
    let mut vendor_ids = String::new();
    for hwd_id in profile.hwd_ids.iter() {
        vendor_ids.push_str(&hwd_id.vendor_ids.join(" "));
        class_ids.push_str(&hwd_id.class_ids.join(" "));
    }

    let desc_formatted = if profile.desc.is_empty() { "-" } else { &profile.desc };

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .add_row(vec![&fl!("name-header"), &profile.name])
        .add_row(vec![&fl!("desc-header"), desc_formatted])
        .add_row(vec![&fl!("priority-header"), &profile.priority.to_string()])
        .add_row(vec![&fl!("nonfree-header"), &profile.is_nonfree.to_string()])
        .add_row(vec![&fl!("classids-header"), &class_ids])
        .add_row(vec![&fl!("vendorids-header"), &vendor_ids]);

    println!("{table}\n");
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::profile::{parse_profiles, HardwareID};

    #[test]
    fn graphics_profiles_correct() {
        let prof_path = "graphic_drivers-profiles-test.toml";
        let parsed_profiles = parse_profiles(prof_path);
        assert!(parsed_profiles.is_ok());

        let hwd_ids = vec![HardwareID {
            class_ids: vec!["0300".to_owned(), "0380".to_owned(), "0302".to_owned()],
            vendor_ids: vec!["10de".to_owned()],
            device_ids: vec!["12".to_owned(), "23".to_owned(), "53".to_owned(), "33".to_owned()],
            blacklisted_class_ids: vec![],
            blacklisted_vendor_ids: vec![],
            blacklisted_device_ids: vec![],
        }];

        let parsed_profiles = parsed_profiles.unwrap();
        assert!(parsed_profiles[0].is_nonfree);
        assert_eq!(parsed_profiles[0].prof_path, prof_path);
        assert_eq!(parsed_profiles[0].name, "nvidia-dkms.40xxcards");
        assert_eq!(
            parsed_profiles[0].desc,
            "Closed source NVIDIA drivers(40xx series) for Linux (Latest)"
        );
        assert_eq!(parsed_profiles[0].priority, 9);
        assert_eq!(
            parsed_profiles[0].packages,
            "nvidia-utils egl-wayland nvidia-settings opencl-nvidia lib32-opencl-nvidia \
             lib32-nvidia-utils libva-nvidia-driver vulkan-icd-loader lib32-vulkan-icd-loader"
        );
        assert_eq!(parsed_profiles[0].device_name_pattern, Some("(AD)\\w+".to_owned()));
        assert_eq!(parsed_profiles[0].hwd_ids, hwd_ids);
        assert!(!parsed_profiles[0].post_install.is_empty());
        assert!(!parsed_profiles[0].post_remove.is_empty());

        assert!(parsed_profiles[1].is_nonfree);
        assert_eq!(parsed_profiles[1].prof_path, prof_path);
        assert_eq!(parsed_profiles[1].name, "nvidia-dkms");
        assert_eq!(parsed_profiles[1].priority, 8);
        assert_eq!(
            parsed_profiles[1].packages,
            "nvidia-utils egl-wayland nvidia-settings opencl-nvidia lib32-opencl-nvidia \
             lib32-nvidia-utils libva-nvidia-driver vulkan-icd-loader lib32-vulkan-icd-loader"
        );
        assert_eq!(parsed_profiles[1].device_name_pattern, None);
        assert_eq!(parsed_profiles[1].hwd_product_name_pattern, Some("(Ally)\\w+".to_owned()));
        assert_eq!(parsed_profiles[1].hwd_ids, hwd_ids);
        assert_eq!(parsed_profiles[1].gc_versions, None);
        assert!(!parsed_profiles[1].post_install.is_empty());
        assert!(!parsed_profiles[1].post_remove.is_empty());
    }

    #[test]
    fn graphics_profiles_invalid() {
        let prof_path = "graphic_drivers-invalid-profiles-test.toml";
        let parsed_profiles = crate::profile::get_invalid_profiles(prof_path);
        assert!(parsed_profiles.is_ok());
        let parsed_profiles = parsed_profiles.unwrap();

        assert_eq!(parsed_profiles.len(), 1);
        assert_eq!(parsed_profiles[0], "nvidia-dkms".to_owned());
    }

    #[test]
    fn profile_write_test() {
        let prof_path = "profile-raw-escaped-strings-test.toml";
        let parsed_profiles = parse_profiles(prof_path);
        assert!(parsed_profiles.is_ok());
        let parsed_profiles = parsed_profiles.unwrap();
        let parsed_profile = &parsed_profiles[0];

        const K_POST_INSTALL_TEST_DATA: &str = r#"    echo "Steam Deck chwd installing..."
    username=$(id -nu 1000)
    services=("steam-powerbuttond")
    kernelparams="amd_iommu=off amdgpu.gttsize=8128 spi_amd.speed_dev=1 audit=0 iomem=relaxed amdgpu.ppfeaturemask=0xffffffff"
    echo "Enabling services..."
    for service in ${services[@]}; do
        systemctl enable --now "${service}.service"
    done
    echo "Adding required kernel parameters..."
    sed -i "s/LINUX_OPTIONS="[^"]*/& ${kernelparams}/" /etc/sdboot-manage.conf
"#;
        assert_eq!(parsed_profile.post_install, K_POST_INSTALL_TEST_DATA);

        // empty file
        let filepath = {
            use std::env;

            let tmp_dir = env::temp_dir();
            format!("{}/.tempfile-chwd-test-{}", tmp_dir.to_string_lossy(), "123451231231")
        };

        assert!(!std::path::Path::new(&filepath).exists());
        assert!(crate::profile::write_profile_to_file(&filepath, parsed_profile));
        let orig_content = fs::read_to_string(&filepath).unwrap();

        // cleanup
        assert!(fs::remove_file(&filepath).is_ok());

        assert_eq!(orig_content, fs::read_to_string(prof_path).unwrap());
    }
}
