fn main() {
    let ctx = libusb::USBContext::new();

    let device_list = ctx.get_device_list().expect("Failed to get device list");

    for device in device_list.iter() {
        let desc = device.device_descriptor().unwrap();
        println!(
            "Bus {:03} Device {:03}: ID {:04x}:{:04x} {}{}{}",
            device.bus_number(),
            device.device_address(),
            desc.idVendor,
            desc.idProduct,
            device.manufacturer(),
            if device.manufacturer().is_empty() || device.product().is_empty() {
                ""
            } else {
                " "
            },
            device.product(),
        );
    }
}
