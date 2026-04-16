#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![no_std]
//! Low-level bindings to the [libusb] library.
//!
//! [libusb]: https://libusb.info/

#[cfg(feature = "std")]
extern crate std;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
