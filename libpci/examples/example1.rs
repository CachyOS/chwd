fn main() {
    let mut pacc = libpci::PCIAccess::new(true);

    let mut i = 1;
    let devices = pacc.devices().expect("Failed");
    for mut item in devices.iter_mut() {
        item.fill_info(libpci::Fill::IDENT as u32 | libpci::Fill::CLASS as u32);
        let item_class = item.class().unwrap();
        let item_vendor = item.vendor().unwrap();
        let item_device = item.device().unwrap();
        println!(
            "class := '{}', vendor := '{}', device := '{}'",
            item_class, item_vendor, item_device
        );
        i += 1;
    }
    println!("i := '{i}'");
}
