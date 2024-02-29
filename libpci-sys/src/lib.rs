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

/// This function returns either ptr to next element or null if data is null.
///
/// # Safety
///
/// The caller must take care of null ptr.
pub unsafe fn pci_get_next_device(data: *const pci_dev) -> *const pci_dev {
    if data.is_null() {
        data
    } else {
        (*data).next
    }
}

/// This function returns either ptr to next element or null if data is null.
///
/// # Safety
///
/// The caller must take care of null ptr.
pub unsafe fn pci_get_next_device_mut(data: *mut pci_dev) -> *mut pci_dev {
    if data.is_null() {
        data
    } else {
        (*data).next
    }
}
