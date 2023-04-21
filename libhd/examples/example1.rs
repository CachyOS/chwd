use libhd;

fn print_vendors(item: libhd::HWItem, scan: i32) {
    let mut hd_data = libhd::HDData::new();
    let hd = libhd::HD::list(&mut hd_data, item, scan, None).expect("Failed to init");

    let mut i = 1;
    for item in hd.iter_mut() {
        let item_vendor = item.vendor().unwrap();
        println!("vendor := '{}'", item_vendor.name);
        i += 1;
    }
    println!("i := '{i}'");
}

fn main() {
    print_vendors(libhd::HWItem::Pci, 1);
    print_vendors(libhd::HWItem::Usb, 1);
}
