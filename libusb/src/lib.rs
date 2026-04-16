#![no_std]

#[cfg(feature = "std")]
extern crate std;

// Re-export libusb-c-sys
pub use libusb_c_sys;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::string::String;

#[cfg(feature = "std")]
use std::string::String;

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(feature = "std")]
use std::format;

#[cfg(feature = "std")]
use std::borrow::ToOwned;

#[cfg(feature = "usb-ids")]
pub mod usb_ids;

use core::marker::PhantomData;
use core::ptr;

/// USB context. Owns the `libusb_context`.
pub struct USBContext {
    handle: *mut libusb_c_sys::libusb_context,
}

unsafe impl Send for USBContext {}

impl Drop for USBContext {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                libusb_c_sys::libusb_exit(self.handle);
            }
        }
    }
}

impl USBContext {
    /// Tries to create a new USB context.
    ///
    /// Returns `None` if `libusb_init` fails.
    pub fn try_new() -> Option<Self> {
        let mut ctx: *mut libusb_c_sys::libusb_context = ptr::null_mut();
        let ret = unsafe { libusb_c_sys::libusb_init(&mut ctx) };
        if ret < 0 || ctx.is_null() {
            None
        } else {
            Some(Self { handle: ctx })
        }
    }

    /// Create a new USB context.
    ///
    /// # Panics
    ///
    /// Panics if `libusb_init` fails.
    pub fn new() -> Self {
        Self::try_new().expect("Failed to initialize libusb")
    }

    /// Get the list of USB devices.
    pub fn get_device_list(&self) -> Option<USBDeviceList<'_>> {
        let mut list: *mut *mut libusb_c_sys::libusb_device = ptr::null_mut();
        let count = unsafe { libusb_c_sys::libusb_get_device_list(self.handle, &mut list) };
        if count < 0 || list.is_null() {
            None
        } else {
            Some(USBDeviceList { list, count: count as usize, _phantom: PhantomData })
        }
    }
}

/// Device list. Owns the array from `libusb_get_device_list`.
pub struct USBDeviceList<'a> {
    list: *mut *mut libusb_c_sys::libusb_device,
    count: usize,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Drop for USBDeviceList<'a> {
    fn drop(&mut self) {
        if !self.list.is_null() {
            // 1 = unref all devices
            unsafe {
                libusb_c_sys::libusb_free_device_list(self.list, 1);
            }
        }
    }
}

impl<'a> USBDeviceList<'a> {
    pub fn iter(&self) -> USBDeviceIter<'a> {
        USBDeviceIter { list: self.list, index: 0, count: self.count, _phantom: PhantomData }
    }
}

pub struct USBDeviceIter<'a> {
    list: *mut *mut libusb_c_sys::libusb_device,
    index: usize,
    count: usize,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Iterator for USBDeviceIter<'a> {
    type Item = USBDevice<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.count {
            let dev = unsafe { *self.list.add(self.index) };
            self.index += 1;
            if !dev.is_null() {
                return Some(USBDevice { dev, _phantom: PhantomData });
            }
        }
        None
    }
}

/// USB device. Wraps a `libusb_device` pointer.
pub struct USBDevice<'a> {
    dev: *mut libusb_c_sys::libusb_device,
    _phantom: PhantomData<&'a ()>,
}

unsafe impl<'a> Send for USBDevice<'a> {}

impl<'a> USBDevice<'a> {
    /// Get the device descriptor.
    pub fn device_descriptor(&self) -> Option<libusb_c_sys::libusb_device_descriptor> {
        if self.dev.is_null() {
            return None;
        }
        let mut desc: libusb_c_sys::libusb_device_descriptor = unsafe { core::mem::zeroed() };
        let ret = unsafe { libusb_c_sys::libusb_get_device_descriptor(self.dev, &mut desc) };
        if ret < 0 {
            None
        } else {
            Some(desc)
        }
    }

    /// Get bus number.
    pub fn bus_number(&self) -> u8 {
        if self.dev.is_null() {
            return 0;
        }
        unsafe { libusb_c_sys::libusb_get_bus_number(self.dev) }
    }

    /// Get device address.
    pub fn device_address(&self) -> u8 {
        if self.dev.is_null() {
            return 0;
        }
        unsafe { libusb_c_sys::libusb_get_device_address(self.dev) }
    }

    /// Get the port number chain from root hub to this device.
    /// Returns an empty vector for root hubs.
    #[cfg(feature = "std")]
    pub fn port_numbers(&self) -> Vec<u8> {
        if self.dev.is_null() {
            return Vec::new();
        }
        // 8 ports is the maximum USB topology depth
        let mut ports = [0u8; 8];
        let count = unsafe {
            libusb_c_sys::libusb_get_port_numbers(
                self.dev,
                ports.as_mut_ptr(),
                ports.len() as core::ffi::c_int,
            )
        };
        if count < 0 {
            Vec::new()
        } else {
            ports[..count as usize].to_vec()
        }
    }

    /// Construct the sysfs bus ID from bus number and port numbers.
    /// E.g. bus=3, ports=[2] → "3-2", bus=1, ports=[1,3] → "1-1.3"
    #[cfg(feature = "std")]
    pub fn sysfs_busid(&self) -> String {
        let bus = self.bus_number();
        let ports = self.port_numbers();
        if ports.is_empty() {
            format!("usb{bus}")
        } else {
            let mut s = format!("{bus}");
            for &port in &ports {
                s.push_str(&format!("-{port}"));
            }
            s
        }
    }

    /// Try to read a sysfs attribute for this device.
    #[cfg(feature = "std")]
    fn read_sysfs_attr(&self, attr: &str) -> Option<String> {
        let busid = self.sysfs_busid();
        let path = format!("/sys/bus/usb/devices/{busid}/{attr}");
        std::fs::read_to_string(&path).ok().map(|s| s.trim().to_owned())
    }

    /// Get a string descriptor as ASCII.
    /// Returns `None` on error (e.g. permission denied).
    fn get_string_descriptor_ascii(&self, desc_index: u8) -> Option<String> {
        if self.dev.is_null() || desc_index == 0 {
            return None;
        }
        let mut handle: *mut libusb_c_sys::libusb_device_handle = ptr::null_mut();
        let ret = unsafe { libusb_c_sys::libusb_open(self.dev, &mut handle) };
        if ret < 0 || handle.is_null() {
            return None;
        }
        let mut buf = [0u8; 256];
        let len = unsafe {
            libusb_c_sys::libusb_get_string_descriptor_ascii(
                handle,
                desc_index,
                buf.as_mut_ptr(),
                buf.len() as core::ffi::c_int,
            )
        };
        unsafe {
            libusb_c_sys::libusb_close(handle);
        }
        if len < 0 {
            None
        } else {
            let slice = &buf[..len as usize];
            Some(String::from(core::str::from_utf8(slice).unwrap_or("")))
        }
    }

    /// Get manufacturer string. Falls back to sysfs when `libusb_open` fails.
    pub fn manufacturer(&self) -> String {
        let desc = match self.device_descriptor() {
            Some(d) => d,
            None => return String::new(),
        };
        if let Some(s) = self.get_string_descriptor_ascii(desc.iManufacturer) {
            return s;
        }
        #[cfg(feature = "std")]
        if let Some(s) = self.read_sysfs_attr("manufacturer") {
            return s;
        }
        String::new()
    }

    /// Get product string. Falls back to sysfs when `libusb_open` fails.
    pub fn product(&self) -> String {
        let desc = match self.device_descriptor() {
            Some(d) => d,
            None => return String::new(),
        };
        if let Some(s) = self.get_string_descriptor_ascii(desc.iProduct) {
            return s;
        }
        #[cfg(feature = "std")]
        if let Some(s) = self.read_sysfs_attr("product") {
            return s;
        }
        String::new()
    }
}
