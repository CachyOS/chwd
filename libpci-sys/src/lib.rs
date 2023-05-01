#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![no_std]
//! Low-level bindings to the [libpci] library.
//!
//! [libpci]: https://mj.ucw.cz/sw/pciutils/

#[cfg(feature = "std")]
extern crate std;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub unsafe fn pci_get_next_device(data: *const pci_dev) -> *const pci_dev {
    if data.is_null() {
        data
    } else {
        (*data).next
    }
}

pub unsafe fn pci_get_next_device_mut(data: *mut pci_dev) -> *mut pci_dev {
    if data.is_null() {
        data
    } else {
        (*data).next
    }
}
