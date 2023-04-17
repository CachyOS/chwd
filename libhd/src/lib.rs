#![no_std]

#[cfg(feature = "std")]
extern crate std;

// Re-export libhd-sys
pub use libhd_sys;

#[cfg(feature = "std")]
use std::os::raw::c_char;

#[cfg(not(feature = "std"))]
use libc::c_char;

use core::marker::PhantomData;
use core::{mem, ptr, str};

/// Returns the LIBHD version.
///
/// Returns `major * 10_000 + minor`.
/// So 22.2 would be returned as `22_002`.
pub fn version_number() -> u32 {
    libhd_sys::HD_FULL_VERSION as u32
}

/// Returns a string representation of the LIBHD version.
///
/// For example "22.2".
pub fn version_string() -> &'static str {
    // Safety: Assumes `hd_version` returns a valid utf8 string.
    unsafe { c_char_to_str(libhd_sys::hd_version()) }
}

#[repr(u32)]
#[derive(Clone, PartialEq)]
pub enum HWItem {
    Nothing = libhd_sys::hw_item_hw_none as u32,
    Sys = libhd_sys::hw_item_hw_sys as u32,
    Cpu = libhd_sys::hw_item_hw_cpu as u32,
    Keyboard = libhd_sys::hw_item_hw_keyboard as u32,
    Braille = libhd_sys::hw_item_hw_braille as u32,
    Mouse = libhd_sys::hw_item_hw_mouse as u32,
    Joystick = libhd_sys::hw_item_hw_joystick as u32,
    Printer = libhd_sys::hw_item_hw_printer as u32,
    Scanner = libhd_sys::hw_item_hw_scanner as u32,
    Chipcard = libhd_sys::hw_item_hw_chipcard as u32,
    Monitor = libhd_sys::hw_item_hw_monitor as u32,
    Tv = libhd_sys::hw_item_hw_tv as u32,
    Display = libhd_sys::hw_item_hw_display as u32,
    Framebuffer = libhd_sys::hw_item_hw_framebuffer as u32,
    Camera = libhd_sys::hw_item_hw_camera as u32,
    Sound = libhd_sys::hw_item_hw_sound as u32,
    StorageCtrl = libhd_sys::hw_item_hw_storage_ctrl as u32,
    NetworkCtrl = libhd_sys::hw_item_hw_network_ctrl as u32,
    Isdn = libhd_sys::hw_item_hw_isdn as u32,
    Modem = libhd_sys::hw_item_hw_modem as u32,
    Network = libhd_sys::hw_item_hw_network as u32,
    Disk = libhd_sys::hw_item_hw_disk as u32,
    Partition = libhd_sys::hw_item_hw_partition as u32,
    Cdrom = libhd_sys::hw_item_hw_cdrom as u32,
    Floppy = libhd_sys::hw_item_hw_floppy as u32,
    Manual = libhd_sys::hw_item_hw_manual as u32,
    UsbCtrl = libhd_sys::hw_item_hw_usb_ctrl as u32,
    Usb = libhd_sys::hw_item_hw_usb as u32,
    Bios = libhd_sys::hw_item_hw_bios as u32,
    Pci = libhd_sys::hw_item_hw_pci as u32,
    Isapnp = libhd_sys::hw_item_hw_isapnp as u32,
    Bridge = libhd_sys::hw_item_hw_bridge as u32,
    Hub = libhd_sys::hw_item_hw_hub as u32,
    Scsi = libhd_sys::hw_item_hw_scsi as u32,
    Ide = libhd_sys::hw_item_hw_ide as u32,
    Memory = libhd_sys::hw_item_hw_memory as u32,
    Dvb = libhd_sys::hw_item_hw_dvb as u32,
    Pcmcia = libhd_sys::hw_item_hw_pcmcia as u32,
    PcmciaCtrl = libhd_sys::hw_item_hw_pcmcia_ctrl as u32,
    Ieee1394 = libhd_sys::hw_item_hw_ieee1394 as u32,
    Ieee1394Ctrl = libhd_sys::hw_item_hw_ieee1394_ctrl as u32,
    Hotplug = libhd_sys::hw_item_hw_hotplug as u32,
    HotplugCtrl = libhd_sys::hw_item_hw_hotplug_ctrl as u32,
    Zip = libhd_sys::hw_item_hw_zip as u32,
    Pppoe = libhd_sys::hw_item_hw_pppoe as u32,
    Wlan = libhd_sys::hw_item_hw_wlan as u32,
    Redasd = libhd_sys::hw_item_hw_redasd as u32,
    Dsl = libhd_sys::hw_item_hw_dsl as u32,
    Block = libhd_sys::hw_item_hw_block as u32,
    Tape = libhd_sys::hw_item_hw_tape as u32,
    Vbe = libhd_sys::hw_item_hw_vbe as u32,
    Bluetooth = libhd_sys::hw_item_hw_bluetooth as u32,
    Fingerprint = libhd_sys::hw_item_hw_fingerprint as u32,
    MmcCtrl = libhd_sys::hw_item_hw_mmc_ctrl as u32,
    Nvme = libhd_sys::hw_item_hw_nvme as u32,
    Unknown = libhd_sys::hw_item_hw_unknown as u32,
    All = libhd_sys::hw_item_hw_all as u32,
}

impl From<u32> for HWItem {
    fn from(value: u32) -> Self {
        if value > HWItem::All as u32 || value < HWItem::Nothing as u32 {
            return HWItem::Nothing;
        }
        unsafe { mem::transmute(value) }
    }
}

/// Individual hardware item.
///
/// Note: Every hardware component gets an \ref hd_t entry. A list of all hardware
/// items is in \ref hd_data_t::hd.
pub struct HD<'a> {
    handle: *mut libhd_sys::hd_t,
    is_list: bool,
    do_drop: bool,
    _phantom: PhantomData<&'a ()>,
}

/// Holds ID + name pairs.
///
/// Note: Used for bus, class, vendor, %device and such.
pub struct HDID {
    pub id: u32,
    pub name: &'static str,
}

/// Holds all data accumulated during hardware probing.
pub struct HDData<'a>(*mut libhd_sys::hd_data_t, PhantomData<&'a ()>);

unsafe impl<'a> Send for HD<'a> {}
unsafe impl<'a> Send for HDData<'a> {}

impl<'a> Drop for HDData<'a> {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }
        // Safety: Just FFI
        unsafe {
            libhd_sys::hd_free_hd_data(self.0);
            libc::free(self.0 as *mut libc::c_void);
        }
    }
}

impl<'a> Drop for HD<'a> {
    fn drop(&mut self) {
        if self.handle.is_null() {
            return;
        }
        // Safety: Just FFI
        unsafe {
            if self.is_list {
                libhd_sys::hd_free_hd_list(self.handle);
            }

            if self.do_drop {
                libc::free(self.handle as *mut libc::c_void);
            }
        }
    }
}

impl Default for HDData<'_> {
    fn default() -> Self {
        HDData::new()
    }
}

impl<'a> HDData<'a> {
    /// Tries to create a new data.
    ///
    /// #### Safety: see [`libc::calloc`]
    ///
    /// Returns `None` if libc::calloc returns a NULL pointer - may happen if allocation fails.
    pub fn try_new() -> Option<Self> {
        Some(HDData(
            unsafe {
                libc::calloc(1, mem::size_of::<libhd_sys::hd_data_t>() as libc::size_t)
                    as *mut libhd_sys::hd_data_t
            },
            PhantomData,
        ))
    }

    pub fn new() -> Self {
        // Safety: Just FFI
        Self::try_new().expect("returned null pointer when allocating memory")
    }
}

impl<'a> HD<'a> {
    /// Tries to create a new data.
    ///
    /// #### Safety: see [`libc::calloc`]
    ///
    /// Returns `None` if libc::calloc returns a NULL pointer - may happen if allocation fails.
    pub fn try_new() -> Option<Self> {
        Some(HD { handle: ptr::null_mut(), is_list: false, do_drop: false, _phantom: PhantomData })
    }

    /// Create a new data.
    ///
    /// # Panics
    ///
    /// Panics if failed to allocate required memory.
    pub fn new() -> Self {
        // Safety: Just FFI
        Self::try_new().expect("returned null pointer when allocating memory")
    }

    /// Constructs from raw C type
    ///
    /// # Safety
    ///
    /// the caller must guarantee that `self` is valid
    /// for a reference if it isn't null.
    pub unsafe fn from_raw(data: *mut libhd_sys::hd_t, take: bool) -> Self {
        HD { handle: data, is_list: take, do_drop: take, _phantom: PhantomData }
    }

    /// Unique index, starting at 1.
    pub fn idx(&self) -> Option<u32> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(inner_obj.idx)
        }
    }

    /// Bus type (id and name).
    pub fn bus(&self) -> Option<HDID> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(HDID { id: inner_obj.bus.id, name: unsafe { c_char_to_str(inner_obj.bus.name) } })
        }
    }

    /// Slot and bus number.
    /// Bits 0-7: slot number, 8-31 bus number.
    pub fn slot(&self) -> Option<u32> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(inner_obj.slot)
        }
    }

    /// (PCI) function.
    pub fn func(&self) -> Option<u32> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(inner_obj.func)
        }
    }

    /// Base class (id and name).
    pub fn base_class(&self) -> Option<HDID> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(HDID {
                id: inner_obj.base_class.id,
                name: unsafe { c_char_to_str(inner_obj.base_class.name) },
            })
        }
    }

    /// Sub class (id and name).
    pub fn sub_class(&self) -> Option<HDID> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(HDID {
                id: inner_obj.sub_class.id,
                name: unsafe { c_char_to_str(inner_obj.sub_class.name) },
            })
        }
    }

    /// (PCI) programming interface (id and name).
    pub fn prog_if(&self) -> Option<HDID> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(HDID {
                id: inner_obj.prog_if.id,
                name: unsafe { c_char_to_str(inner_obj.prog_if.name) },
            })
        }
    }

    /// Vendor id and name.
    ///
    /// Id is actually a combination of some tag to differentiate the
    /// various id types and the real id.
    pub fn vendor(&self) -> Option<HDID> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(HDID {
                id: inner_obj.vendor.id,
                name: unsafe { c_char_to_str(inner_obj.vendor.name) },
            })
        }
    }

    /// Device id and name.
    ///
    /// Id is actually a combination of some tag to differentiate the
    /// various id types and the real id.
    ///
    /// Note: If you're looking or something printable, you might want to use [`crate::HD::model`]
    /// instead.
    pub fn device(&self) -> Option<HDID> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(HDID {
                id: inner_obj.device.id,
                name: unsafe { c_char_to_str(inner_obj.device.name) },
            })
        }
    }

    /// Subvendor id and name.
    ///
    /// Id is actually a combination of some tag to differentiate the
    /// various id types and the real id.
    pub fn sub_vendor(&self) -> Option<HDID> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(HDID {
                id: inner_obj.sub_vendor.id,
                name: unsafe { c_char_to_str(inner_obj.sub_vendor.name) },
            })
        }
    }

    /// Subdevice id and name.
    ///
    /// Id is actually a combination of some tag to differentiate the
    /// various id types and the real id.
    pub fn sub_device(&self) -> Option<HDID> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(HDID {
                id: inner_obj.sub_device.id,
                name: unsafe { c_char_to_str(inner_obj.sub_device.name) },
            })
        }
    }

    /// Revision id or string.
    ///
    /// Note:
    ///
    /// - If revision *is* numerical (e.g. PCI) [`crate::HDID::id`] is used.
    /// - If revision *is* some char data (e.g. disk drives) it is stored in [`crate::HDID::name`].
    pub fn revision(&self) -> Option<HDID> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(HDID {
                id: inner_obj.revision.id,
                name: unsafe { c_char_to_str(inner_obj.revision.name) },
            })
        }
    }

    /// Serial id.
    pub fn serial(&self) -> Option<&'static str> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(unsafe { c_char_to_str(inner_obj.serial) })
        }
    }

    /// Vendor id and name of some compatible hardware.
    ///
    /// Note: Used mainly for ISA-PnP devices.
    pub fn compat_vendor(&self) -> Option<HDID> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(HDID {
                id: inner_obj.compat_vendor.id,
                name: unsafe { c_char_to_str(inner_obj.compat_vendor.name) },
            })
        }
    }

    /// Device id and name of some compatible hardware.
    ///
    /// Note: Used mainly for ISA-PnP devices.
    pub fn compat_device(&self) -> Option<HDID> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(HDID {
                id: inner_obj.compat_device.id,
                name: unsafe { c_char_to_str(inner_obj.compat_device.name) },
            })
        }
    }

    /// Hardware class.
    ///
    /// Note: Not to confuse with [`crate::HD::base_class`]!
    pub fn hw_class(&self) -> Option<u32> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(inner_obj.hw_class)
        }
    }

    /// Model name.
    ///
    /// Note: This is a combination of vendor and device names. Some heuristics is used
    /// to make it more presentable. Use this instead of [`crate::HD::vendor`] and
    /// [`crate::HD::device`].
    pub fn model(&self) -> Option<&'static str> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(unsafe { c_char_to_str(inner_obj.model) })
        }
    }

    /// Device this hardware is attached to.
    ///
    /// Note: Link to some 'parent' %device. Use \ref hd_get_device_by_idx() to get
    /// the corresponding hardware entry.
    pub fn attached_to(&self) -> Option<u32> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(inner_obj.attached_to)
        }
    }

    /// sysfs entry for this hardware, if any.
    pub fn sysfs_id(&self) -> Option<&'static str> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(unsafe { c_char_to_str(inner_obj.sysfs_id) })
        }
    }

    /// sysfs bus id for this hardware, if any.
    pub fn sysfs_bus_id(&self) -> Option<&'static str> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(unsafe { c_char_to_str(inner_obj.sysfs_bus_id) })
        }
    }

    /// sysfs device link.
    pub fn sysfs_device_link(&self) -> Option<&'static str> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(unsafe { c_char_to_str(inner_obj.sysfs_device_link) })
        }
    }

    /// Special %device file.
    ///
    /// Note: Device file name to access this hardware.
    pub fn unix_dev_name(&self) -> Option<&'static str> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(unsafe { c_char_to_str(inner_obj.unix_dev_name) })
        }
    }

    /// Special %device file.
    ///
    /// Note: Device file name to access this hardware.
    pub fn unix_dev_name2(&self) -> Option<&'static str> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(unsafe { c_char_to_str(inner_obj.unix_dev_name2) })
        }
    }

    /// BIOS/PROM id.
    ///
    /// Note: Where appropriate, this is a special BIOS/PROM id (e.g. "0x80" for
    /// the first harddisk on Intel-PCs).
    /// CHPID for s390.
    pub fn rom_id(&self) -> Option<&'static str> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(unsafe { c_char_to_str(inner_obj.rom_id) })
        }
    }

    /// HAL udi.
    pub fn udi(&self) -> Option<&'static str> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(unsafe { c_char_to_str(inner_obj.udi) })
        }
    }

    /// [`crate::HD::udi`] of parent ([`crate::HD::attached_to`])
    pub fn parent_udi(&self) -> Option<&'static str> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(unsafe { c_char_to_str(inner_obj.parent_udi) })
        }
    }

    /// Unique id for this hardware.
    ///
    /// Note: A unique string identifying this hardware. The string consists
    /// of two parts separated by a dot ("."). The part before the dot
    /// describes the location (where the hardware is attached in the system).
    /// The part after the dot identifies the hardware itself. The string
    /// must not contain slashes ("/") because we're going to create files
    /// with this id as name. Apart from this there are no restrictions on
    /// the form of this string.
    pub fn unique_id(&self) -> Option<&'static str> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(unsafe { c_char_to_str(inner_obj.unique_id) })
        }
    }

    /// Currently active driver.
    pub fn driver(&self) -> Option<&'static str> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(unsafe { c_char_to_str(inner_obj.driver) })
        }
    }

    /// Currently active driver module (if any).
    pub fn driver_module(&self) -> Option<&'static str> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(unsafe { c_char_to_str(inner_obj.driver_module) })
        }
    }

    /// [`crate::HD::unique_id`] of parent ([`crate::HD::attached_to`])
    pub fn parent_id(&self) -> Option<&'static str> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(unsafe { c_char_to_str(inner_obj.parent_id) })
        }
    }

    /// USB Global Unique Identifier.
    ///
    /// Note: Available for USB devices. This may even be set if [`crate::HD::bus`] is not
    /// bus_usb (e.g. USB storage devices will have [`crate::HD::bus`] set to
    /// bus_scsi due to SCSI emulation).
    pub fn usb_guid(&self) -> Option<&'static str> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(unsafe { c_char_to_str(inner_obj.usb_guid) })
        }
    }

    /// module alias
    pub fn modalias(&self) -> Option<&'static str> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(unsafe { c_char_to_str(inner_obj.modalias) })
        }
    }

    /// Consistent Device Name (CDN), pci firmware spec 3.1, chapter 4.6.7
    pub fn label(&self) -> Option<&'static str> {
        if self.handle.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.handle };
            Some(unsafe { c_char_to_str(inner_obj.label) })
        }
    }

    pub fn list(
        data: &mut HDData,
        item: HWItem,
        rescan: i32,
        hd_old: Option<&mut Self>,
    ) -> Option<Self> {
        let hd_old_cov = if hd_old.is_none() { ptr::null_mut() } else { hd_old.unwrap().handle };
        Some(HD {
            handle: unsafe { libhd_sys::hd_list(data.0, item as u32, rescan, hd_old_cov) },
            is_list: true,
            do_drop: false,
            _phantom: PhantomData,
        })
    }

    pub fn dump_entry(data: &mut HDData, hd: &mut Self) {
        unsafe { libhd_sys::hd_dump_entry(data.0, hd.handle, libhd_sys::get_stdout_ptr()) }
    }

    pub fn len(hd: &Self) -> usize {
        unsafe { libhd_sys::hd_get_len(hd.handle) }
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter { ptr: self.handle as _, _phantom: PhantomData }
    }

    pub fn iter_mut(&self) -> IterMut<'_> {
        IterMut { ptr: self.handle as _, _phantom: PhantomData }
    }
}

pub struct Iter<'a> {
    ptr: *const libhd_sys::hd_t,
    _phantom: PhantomData<&'a ()>,
}

pub struct IterMut<'a> {
    ptr: *mut libhd_sys::hd_t,
    _phantom: PhantomData<&'a mut ()>,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = HD<'a>;

    // next() is the only required method
    fn next(&mut self) -> Option<HD<'a>> {
        let ptr = self.ptr;
        self.ptr = unsafe { libhd_sys::hd_get_next_entry_mut(self.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { HD::from_raw(ptr, false) })
        }
    }
}

unsafe fn c_char_to_str(text: *const c_char) -> &'static str {
    if text.is_null() {
        return "";
    }
    #[cfg(not(feature = "std"))]
    {
        // To be safe, we need to compute right now its length
        let len = libc::strlen(text);
        // Cast it to a slice
        let slice = core::slice::from_raw_parts(text as *mut u8, len);
        // And hope it's still text.
        str::from_utf8(slice).expect("bad error message from hd")
    }

    #[cfg(feature = "std")]
    {
        std::ffi::CStr::from_ptr(text).to_str().expect("bad error message from hd")
    }
}
