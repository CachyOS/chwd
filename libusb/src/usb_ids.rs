use std::borrow::ToOwned;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::string::String;

/// USB IDs database parsed from `usb.ids` (e.g. `/usr/share/hwdata/usb.ids`).
pub struct UsbIds {
    vendors: HashMap<u16, String>,
    products: HashMap<(u16, u16), String>,
}

impl UsbIds {
    /// Search paths for the USB IDs database, in priority order.
    const PATHS: &'static [&'static str] =
        &["/usr/share/hwdata/usb.ids", "/usr/share/misc/usb.ids", "/var/lib/usbutils/usb.ids"];

    /// Load the USB IDs database from the first found default path.
    pub fn load() -> Option<Self> {
        for path in Self::PATHS {
            if Path::new(path).exists() {
                if let Some(db) = Self::load_from(path) {
                    return Some(db);
                }
            }
        }
        None
    }

    /// Load the USB IDs database from a specific file path.
    pub fn load_from(path: &str) -> Option<Self> {
        let contents = fs::read_to_string(path).ok()?;
        Some(Self::parse(&contents))
    }

    fn parse(contents: &str) -> Self {
        let mut vendors = HashMap::new();
        let mut products = HashMap::new();
        let mut current_vendor: Option<u16> = None;

        for line in contents.lines() {
            // Skip comments and blank lines
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            let trimmed = line.trim_start();

            // Skip lines that are not vendor or device entries
            if line.starts_with('\t') {
                // Device line: \tPPPP  Product Name (one tab = device)
                if !line.starts_with("\t\t") {
                    if let Some(vendor_id) = current_vendor {
                        if let Some((id, name)) = parse_id_entry(trimmed) {
                            products.insert((vendor_id, id), name);
                        }
                    }
                }
                // Two tabs = interface, skip
            } else {
                // Vendor line: VVVV  Vendor Name
                if let Some((id, name)) = parse_id_entry(line) {
                    vendors.insert(id, name);
                    current_vendor = Some(id);
                } else {
                    current_vendor = None;
                }
            }
        }

        Self { vendors, products }
    }

    /// Look up a vendor name by vendor ID.
    pub fn vendor_name(&self, vendor_id: u16) -> Option<&str> {
        self.vendors.get(&vendor_id).map(|s| s.as_str())
    }

    /// Look up a product name by vendor ID and product ID.
    pub fn product_name(&self, vendor_id: u16, product_id: u16) -> Option<&str> {
        self.products.get(&(vendor_id, product_id)).map(|s| s.as_str())
    }
}

/// Parse a single `ID  Name` line. Returns `(id, name)`.
fn parse_id_entry(line: &str) -> Option<(u16, String)> {
    // Format: "VVVV  Vendor Name" or "PPPP  Product Name"
    // At least two spaces separate the hex ID from the name
    let rest = line.trim_start();
    let hex_str = rest.get(..4)?;
    let id = u16::from_str_radix(hex_str, 16).ok()?;
    // Skip hex digits and the separator (2+ spaces or whitespace)
    let name_part = rest.get(4..)?.trim_start();
    Some((id, name_part.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let db = UsbIds::parse(
            "\
# test
1d6b  Linux Foundation
\t0002  2.0 root hub
\t0003  3.0 root hub
0b05  ASUSTek Computer, Inc.
\t190e  ASUS USB-BT500
",
        );
        assert_eq!(db.vendor_name(0x1d6b), Some("Linux Foundation"));
        assert_eq!(db.vendor_name(0x0b05), Some("ASUSTek Computer, Inc."));
        assert_eq!(db.product_name(0x1d6b, 0x0002), Some("2.0 root hub"));
        assert_eq!(db.product_name(0x0b05, 0x190e), Some("ASUS USB-BT500"));
        assert_eq!(db.vendor_name(0xffff), None);
        assert_eq!(db.product_name(0x0b05, 0x0000), None);
    }
}
