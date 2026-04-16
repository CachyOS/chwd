fn main() {
    let ctx = libusb::USBContext::new();

    let device_list = ctx.get_device_list().expect("Failed to get device list");

    let usb_ids = libusb::usb_ids::UsbIds::load();

    let mut devices: Vec<_> = device_list.iter().collect::<Vec<_>>();
    devices.sort_by_key(|d| (d.bus_number(), d.device_address()));

    for device in &devices {
        let desc = device.device_descriptor().unwrap();

        let vendor_name = device.resolved_vendor_name(&desc, usb_ids.as_ref());
        let product_name = device.resolved_product_name(&desc, usb_ids.as_ref());

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
