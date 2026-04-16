fn main() {
    let ctx = libusb::USBContext::new();

    let device_list = ctx.get_device_list().expect("Failed to get device list");

    let usb_ids = libusb::usb_ids::UsbIds::load();

    let mut devices: Vec<_> = device_list.iter().collect::<Vec<_>>();
    devices.sort_by_key(|d| (d.bus_number(), d.device_address()));

    for device in &devices {
        let desc = device.device_descriptor().unwrap();

        // Try USB IDs database first, fall back to sysfs
        let vendor_name = usb_ids
            .as_ref()
            .and_then(|db| db.vendor_name(desc.idVendor).map(|s| s.to_owned()))
            .unwrap_or_else(|| device.manufacturer());
        let product_name = usb_ids
            .as_ref()
            .and_then(|db| db.product_name(desc.idVendor, desc.idProduct).map(|s| s.to_owned()))
            .unwrap_or_else(|| device.product());

        println!(
            "Bus {:03} Device {:03}: ID {:04x}:{:04x} {}{}{}",
            device.bus_number(),
            device.device_address(),
            desc.idVendor,
            desc.idProduct,
            vendor_name,
            if vendor_name.is_empty() || product_name.is_empty() { "" } else { " " },
            product_name,
        );
    }
}
