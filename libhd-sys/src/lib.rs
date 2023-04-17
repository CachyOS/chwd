#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![no_std]
//! Low-level bindings to the [libhd] library.
//!
//! [libhd]: https://github.com/openSUSE/hwinfo/

#[cfg(feature = "std")]
extern crate std;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub unsafe fn hd_get_next_entry(data: *const hd_t) -> *const hd_t {
    if data.is_null() {
        data
    } else {
        (*data).next
    }
}

pub unsafe fn hd_get_next_entry_mut(data: *mut hd_t) -> *mut hd_t {
    if data.is_null() {
        data
    } else {
        (*data).next
    }
}

pub unsafe fn hd_get_len(data: *mut hd_t) -> usize {
    let mut iter = data;
    let mut i = 0;
    loop {
        if iter.is_null() {
            break;
        }
        i += 1;
        iter = hd_get_next_entry_mut(iter);
    }

    i
}

extern "C" {
    pub fn get_stdout_ptr() -> *mut libc::FILE;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "std")]
    use std::mem;

    #[cfg(not(feature = "std"))]
    use core::mem;

    #[test]
    fn list_hardware() {
        unsafe {
            let mut hd_data: hd_data_t = mem::zeroed();
            let hd_list_obj =
                hd_list(&mut hd_data as *mut _, hw_item_hw_pci, 1 as _, mem::zeroed());

            let mut iter = hd_list_obj;
            let mut i = 0;
            loop {
                if iter.is_null() {
                    break;
                }
                i += 1;
                iter = hd_get_next_entry_mut(iter);
            }

            assert!(i > 0);

            hd_free_hd_list(hd_list_obj);
            hd_free_hd_data(&mut hd_data as *mut _);
        }
    }
}
