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
use crate::profile::Profile;

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};

pub fn print_profile_details(profile: &Profile) {
    let mut class_ids = String::new();
    let mut vendor_ids = String::new();
    for hwd_id in &profile.hwd_ids {
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
        .add_row(vec![&fl!("classids-header"), &class_ids])
        .add_row(vec![&fl!("vendorids-header"), &vendor_ids]);

    println!("{table}\n");
}
