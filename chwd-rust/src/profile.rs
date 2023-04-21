// Copyright (C) 2023 Vladislav Nepogodin
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

use crate::ffi::Profile;

use anyhow::Result;
use std::fs;

impl Profile {
    pub fn new() -> Self {
        Self {
            is_nonfree: false,
            prof_path: "".to_owned(),
            prof_type: "".to_owned(),
            name: "".to_owned(),
            desc: "".to_owned(),
            priority: 0,
            hwd_ids: Vec::from([Default::default()]),
        }
    }
}

pub fn parse_profiles(file_path: &str, type_name: &str) -> Result<Vec<Profile>> {
    let mut profiles = vec![];
    let file_content = fs::read_to_string(file_path)?;
    let toml_table = file_content.parse::<toml::Table>()?;

    for (key, value) in toml_table.iter() {
        if !value.is_table() {
            continue;
        }

        let toplevel_profile = parse_profile(value.as_table().unwrap(), key);
        if toplevel_profile.is_err() {
            continue;
        }

        for (nested_key, nested_value) in value.as_table().unwrap().iter() {
            if !nested_value.is_table() {
                continue;
            }
            let nested_profile_name = format!("{}.{}", key, nested_key);
            let mut nested_value_table = nested_value.as_table().unwrap().clone();
            merge_table_left(&mut nested_value_table, value.as_table().unwrap());
            let nested_profile = parse_profile(&nested_value_table, &nested_profile_name);
            if nested_profile.is_err() {
                continue;
            }
            let mut nested_profile = nested_profile?;
            nested_profile.prof_type = type_name.to_owned();
            nested_profile.prof_path = file_path.to_owned();
            profiles.push(nested_profile);
        }
        let mut toplevel_profile = toplevel_profile?;
        toplevel_profile.prof_type = type_name.to_owned();
        toplevel_profile.prof_path = file_path.to_owned();
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

        let toplevel_profile = parse_profile(value.as_table().unwrap(), key);
        if toplevel_profile.is_err() {
            invalid_profile_list.push(key.to_owned());
            continue;
        }

        for (nested_key, nested_value) in value.as_table().unwrap().iter() {
            if !nested_value.is_table() {
                continue;
            }
            let nested_profile_name = format!("{}.{}", key, nested_key);
            let mut nested_value_table = nested_value.as_table().unwrap().clone();
            merge_table_left(&mut nested_value_table, value.as_table().unwrap());
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
        is_nonfree: node.get("nonfree").and_then(|x| x.as_bool()).unwrap_or(false).to_owned(),
        prof_path: "".to_owned(),
        prof_type: "".to_owned(),
        name: profile_name.to_owned(),
        desc: node.get("desc").and_then(|x| x.as_str()).unwrap_or("").to_owned(),
        priority: node.get("priority").and_then(|x| x.as_integer()).unwrap_or(0) as i32,
        hwd_ids: Vec::from([Default::default()]),
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
    profile.hwd_ids.last_mut().unwrap().device_ids = devids_val
        .split(' ')
        .into_iter()
        .filter(|x| !x.is_empty())
        .map(|x| x.to_owned())
        .collect::<Vec<_>>();
    if !profile.hwd_ids.last().unwrap().class_ids.is_empty() {
        profile.hwd_ids.push(Default::default());
    }
    profile.hwd_ids.last_mut().unwrap().class_ids = conf_classids
        .split(' ')
        .into_iter()
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
            .into_iter()
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
    let file_content = fs::read_to_string(file_path)?;
    let parsed_ids = file_content
        .lines()
        .filter(|x| !x.trim().is_empty() && x.trim().as_bytes()[0] != b'#')
        .map(|x| format!(" {}", x.trim()))
        .collect::<String>();

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
    table.insert("desc".to_owned(), profile.desc.clone().into());
    table.insert("priority".to_owned(), profile.priority.into());

    let device_ids = profile.hwd_ids.last().unwrap().device_ids.clone();
    let vendor_ids = profile.hwd_ids.last().unwrap().vendor_ids.clone();
    let class_ids = profile.hwd_ids.last().unwrap().class_ids.clone();
    table.insert("device_ids".to_owned(), device_ids.join(" ").into());
    table.insert("vendor_ids".to_owned(), vendor_ids.join(" ").into());
    table.insert("class_ids".to_owned(), class_ids.join(" ").into());

    let toml_string = format!("[{}]\n{}", profile.name, toml::to_string(&table).unwrap());
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

    println!(
        "   NAME:\t{}\n   ATTACHED:\t{}\n   INFO:\t{}\n   PRIORITY:\t{}\n   NONFREE:\t{}\n   \
         CLASSIDS:\t{}\n   VENDORIDS:\t{}\n",
        profile.name,
        profile.prof_type,
        desc_formatted,
        profile.priority,
        profile.is_nonfree,
        class_ids,
        vendor_ids
    );
}
