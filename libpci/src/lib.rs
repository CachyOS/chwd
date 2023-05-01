#![no_std]

#[cfg(feature = "std")]
extern crate std;

// Re-export libpci-sys
pub use libpci_sys;

#[cfg(feature = "std")]
use std::os::raw::c_char;

#[cfg(not(feature = "std"))]
use libc::c_char;

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

use core::marker::PhantomData;
use core::{mem, ptr, str};
use alloc::{vec::Vec, string::String};

/// Returns the LIBHD version.
///
/// Returns `major * 10_000 + minor`.
/// So 22.2 would be returned as `22_002`.
pub fn version_number() -> u32 {
    libpci_sys::PCI_LIB_VERSION as u32
}

#[repr(u32)]
#[derive(Clone, PartialEq)]
pub enum Fill {
    IDENT = libpci_sys::PCI_FILL_IDENT as u32,
    IRQ = libpci_sys::PCI_FILL_IRQ as u32,
    BASES = libpci_sys::PCI_FILL_BASES as u32,
    RomBase = libpci_sys::PCI_FILL_ROM_BASE as u32,
    SIZES = libpci_sys::PCI_FILL_SIZES as u32,
    CLASS = libpci_sys::PCI_FILL_CLASS as u32,
    CAPS = libpci_sys::PCI_FILL_CAPS as u32,
    ExtCaps = libpci_sys::PCI_FILL_EXT_CAPS as u32,
    PhysSlot = libpci_sys::PCI_FILL_PHYS_SLOT as u32,
    ModuleAlias = libpci_sys::PCI_FILL_MODULE_ALIAS as u32,
    LABEL = libpci_sys::PCI_FILL_LABEL as u32,
    NumaNode = libpci_sys::PCI_FILL_NUMA_NODE as u32,
    IoFlags = libpci_sys::PCI_FILL_IO_FLAGS as u32,
    DtNode = libpci_sys::PCI_FILL_DT_NODE as u32,
    IommuGroup = libpci_sys::PCI_FILL_IOMMU_GROUP as u32,
    BridgeBases = libpci_sys::PCI_FILL_BRIDGE_BASES as u32,
    RESCAN = libpci_sys::PCI_FILL_RESCAN as u32,
    ClassExt = libpci_sys::PCI_FILL_CLASS_EXT as u32,
    SUBSYS = libpci_sys::PCI_FILL_SUBSYS as u32,
    PARENT = libpci_sys::PCI_FILL_PARENT as u32,
    DRIVER = libpci_sys::PCI_FILL_DRIVER as u32,
}

#[repr(u32)]
#[derive(Clone, PartialEq)]
pub enum AccessType {
    Auto = libpci_sys::pci_access_type_PCI_ACCESS_AUTO as u32,
    SysBusPci = libpci_sys::pci_access_type_PCI_ACCESS_SYS_BUS_PCI as u32,
    ProcBusPci = libpci_sys::pci_access_type_PCI_ACCESS_PROC_BUS_PCI as u32,
    I386Type1 = libpci_sys::pci_access_type_PCI_ACCESS_I386_TYPE1 as u32,
    I386Type2 = libpci_sys::pci_access_type_PCI_ACCESS_I386_TYPE2 as u32,
    FbsdDevice = libpci_sys::pci_access_type_PCI_ACCESS_FBSD_DEVICE as u32,
    AixDevice = libpci_sys::pci_access_type_PCI_ACCESS_AIX_DEVICE as u32,
    NbsdLibpci = libpci_sys::pci_access_type_PCI_ACCESS_NBSD_LIBPCI as u32,
    ObsdDevice = libpci_sys::pci_access_type_PCI_ACCESS_OBSD_DEVICE as u32,
    Dump = libpci_sys::pci_access_type_PCI_ACCESS_DUMP as u32,
    Darwin = libpci_sys::pci_access_type_PCI_ACCESS_DARWIN as u32,
    SylixosDevice = libpci_sys::pci_access_type_PCI_ACCESS_SYLIXOS_DEVICE as u32,
    HURD = libpci_sys::pci_access_type_PCI_ACCESS_HURD as u32,
    Win32Cfgmgr32 = libpci_sys::pci_access_type_PCI_ACCESS_WIN32_CFGMGR32 as u32,
    Win32Kldbg = libpci_sys::pci_access_type_PCI_ACCESS_WIN32_KLDBG as u32,
    Win32Sysdbg = libpci_sys::pci_access_type_PCI_ACCESS_WIN32_SYSDBG as u32,
    MmioType1 = libpci_sys::pci_access_type_PCI_ACCESS_MMIO_TYPE1 as u32,
    Type1Ext = libpci_sys::pci_access_type_PCI_ACCESS_MMIO_TYPE1_EXT as u32,
    Max = libpci_sys::pci_access_type_PCI_ACCESS_MAX as u32,
}

impl From<u32> for AccessType {
    fn from(value: u32) -> Self {
        if value > AccessType::Max as u32 || value < AccessType::Auto as u32 {
            return AccessType::Auto;
        }
        unsafe { mem::transmute(value) }
    }
}

/// Individual hardware item.
///
/// Note: Every hardware component gets an \ref hd_t entry. A list of all hardware
/// items is in \ref hd_data_t::hd.
#[derive(Clone, Debug)]
pub struct PCIAccess<'a> {
    handle: *mut libpci_sys::pci_access,
    _phantom: PhantomData<&'a ()>,
}

// /// Holds ID + name pairs.
// ///
// /// Note: Used for bus, class, vendor, %device and such.
// pub struct HDID {
//     pub id: u32,
//     pub name: &'static str,
// }

/// Holds all data accumulated during hardware probing.
pub struct PCIDevice<'a>(*mut libpci_sys::pci_dev, PhantomData<&'a ()>);

unsafe impl<'a> Send for PCIAccess<'a> {}
unsafe impl<'a> Send for PCIDevice<'a> {}

impl<'a> Drop for PCIDevice<'a> {
    fn drop(&mut self) {
        //if self.0.is_null() {
        //    return;
        //}
        // Safety: Just FFI
        //unsafe {
        //    libhd_sys::hd_free_hd_data(self.0);
        //    libc::free(self.0 as *mut libc::c_void);
        //}
    }
}

impl<'a> Drop for PCIAccess<'a> {
    fn drop(&mut self) {
        if self.handle.is_null() {
            return;
        }
        // Safety: Just FFI
        unsafe {
            libpci_sys::pci_cleanup(self.handle);
        }
    }
}

// impl Default for HDData<'_> {
//     fn default() -> Self {
//         HDData::new()
//     }
// }

impl<'a> PCIAccess<'a> {
    /// Tries to create a new data.
    ///
    /// Safety: Just FFI
    ///
    /// Returns `None` if libpci_sys::pci_alloc returns a NULL pointer - may happen if allocation fails.
    pub fn try_new(do_scan: bool) -> Option<Self> {
        let ptr: *mut libpci_sys::pci_access = unsafe { libpci_sys::pci_alloc() };
        if ptr.is_null() {
            None
        } else {
            unsafe { libpci_sys::pci_init(ptr); }
            if do_scan {
                unsafe { libpci_sys::pci_scan_bus(ptr); }
            }
            Some(Self { handle: ptr, _phantom: PhantomData })
        }
    }

    /// Create a new data.
    ///
    /// # Panics
    ///
    /// Panics if failed to allocate required memory.
    pub fn new(do_scan: bool) -> Self {
        // Safety: Just FFI
        Self::try_new(do_scan).expect("returned null pointer when allocating memory")
    }

    /// Scan to get the list of devices
    pub fn scan_bus(&mut self) {
        if !self.handle.is_null() {
            unsafe { libpci_sys::pci_scan_bus(self.handle); }
        }
    }

    /// Get linked list of devices
    pub fn devices(&mut self) -> Option<PCIDevice> {
        if self.handle.is_null() {
            None
        } else {
            Some( unsafe { PCIDevice::from_raw((*self.handle).devices) } )
        }
    }
}


impl<'a> PCIDevice<'a> {
    /// Tries to create a new data.
    ///
    /// Safety: Just FFI
    ///
    /// Returns `None` if libpci_sys::pci_alloc returns a NULL pointer - may happen if allocation fails.
    pub fn try_new() -> Option<Self> {
        Some(Self ( ptr::null_mut::<libpci_sys::pci_dev>(), PhantomData ))
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
    pub unsafe fn from_raw(data: *mut libpci_sys::pci_dev) -> Self {
        Self ( data, PhantomData )
    }


    /// Scan to get the list of devices
    pub fn fill_info(&mut self, fill: u32) {
        if !self.0.is_null() {
            unsafe { libpci_sys::pci_fill_info(self.0, fill as _); }
        }
    }

//     /// Bus type (id and name).
//     pub fn bus(&self) -> Option<HDID> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(HDID { id: inner_obj.bus.id, name: unsafe { c_char_to_str(inner_obj.bus.name) } })
//         }
//     }

//     /// Slot and bus number.
//     /// Bits 0-7: slot number, 8-31 bus number.
//     pub fn slot(&self) -> Option<u32> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(inner_obj.slot)
//         }
//     }

//     /// (PCI) function.
//     pub fn func(&self) -> Option<u32> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(inner_obj.func)
//         }
//     }

//     /// Base class (id and name).
//     pub fn base_class(&self) -> Option<HDID> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(HDID {
//                 id: inner_obj.base_class.id,
//                 name: unsafe { c_char_to_str(inner_obj.base_class.name) },
//             })
//         }
//     }

//     /// Sub class (id and name).
//     pub fn sub_class(&self) -> Option<HDID> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(HDID {
//                 id: inner_obj.sub_class.id,
//                 name: unsafe { c_char_to_str(inner_obj.sub_class.name) },
//             })
//         }
//     }

//     /// (PCI) programming interface (id and name).
//     pub fn prog_if(&self) -> Option<HDID> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(HDID {
//                 id: inner_obj.prog_if.id,
//                 name: unsafe { c_char_to_str(inner_obj.prog_if.name) },
//             })
//         }
//     }

//     /// Vendor id and name.
//     ///
//     /// Id is actually a combination of some tag to differentiate the
//     /// various id types and the real id.
//     pub fn vendor(&self) -> Option<HDID> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(HDID {
//                 id: inner_obj.vendor.id,
//                 name: unsafe { c_char_to_str(inner_obj.vendor.name) },
//             })
//         }
//     }

//     /// Device id and name.
//     ///
//     /// Id is actually a combination of some tag to differentiate the
//     /// various id types and the real id.
//     ///
//     /// Note: If you're looking or something printable, you might want to use [`crate::HD::model`]
//     /// instead.
//     pub fn device(&self) -> Option<HDID> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(HDID {
//                 id: inner_obj.device.id,
//                 name: unsafe { c_char_to_str(inner_obj.device.name) },
//             })
//         }
//     }

//     /// Subvendor id and name.
//     ///
//     /// Id is actually a combination of some tag to differentiate the
//     /// various id types and the real id.
//     pub fn sub_vendor(&self) -> Option<HDID> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(HDID {
//                 id: inner_obj.sub_vendor.id,
//                 name: unsafe { c_char_to_str(inner_obj.sub_vendor.name) },
//             })
//         }
//     }

//     /// Subdevice id and name.
//     ///
//     /// Id is actually a combination of some tag to differentiate the
//     /// various id types and the real id.
//     pub fn sub_device(&self) -> Option<HDID> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(HDID {
//                 id: inner_obj.sub_device.id,
//                 name: unsafe { c_char_to_str(inner_obj.sub_device.name) },
//             })
//         }
//     }

//     /// Revision id or string.
//     ///
//     /// Note:
//     ///
//     /// - If revision *is* numerical (e.g. PCI) [`crate::HDID::id`] is used.
//     /// - If revision *is* some char data (e.g. disk drives) it is stored in [`crate::HDID::name`].
//     pub fn revision(&self) -> Option<HDID> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(HDID {
//                 id: inner_obj.revision.id,
//                 name: unsafe { c_char_to_str(inner_obj.revision.name) },
//             })
//         }
//     }

    /// Get class str.
    pub fn class(&self) -> Option<String> {
        if self.0.is_null() {
            None
        } else {
            let mut class = vec![0 as u8; 1024];
            let size = (class.len() * mem::size_of::<u8>()) as usize;

            unsafe { libpci_sys::pci_lookup_class_helper((*self.0).access, class.as_mut_ptr() as _, size, self.0); }
            Some(String::from(str::from_utf8(&class).ok()?))
        }
    }

    /// Get vendor str.
    pub fn vendor(&self) -> Option<String> {
        if self.0.is_null() {
            None
        } else {
            let mut vendor = vec![0 as u8; 256];
            let size = (vendor.len() * mem::size_of::<u8>()) as usize;

            unsafe { libpci_sys::pci_lookup_vendor_helper((*self.0).access, vendor.as_mut_ptr() as _, size, self.0); }
            Some(String::from(str::from_utf8(&vendor).ok()?))
        }
    }

    /// Get device str.
    pub fn device(&self) -> Option<String> {
        if self.0.is_null() {
            None
        } else {
            let mut device = vec![0 as u8; 256];
            let size = (device.len() * mem::size_of::<u8>()) as usize;

            unsafe { libpci_sys::pci_lookup_device_helper((*self.0).access, device.as_mut_ptr() as _, size, self.0); }
            Some(String::from(str::from_utf8(&device).ok()?))
        }
    }

    /// Class ID.
    pub fn class_id(&self) -> Option<u16> {
        if self.0.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.0 };
            Some(inner_obj.device_class)
        }
    }

    /// Vendor ID.
    pub fn vendor_id(&self) -> Option<u16> {
        if self.0.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.0 };
            Some(inner_obj.vendor_id)
        }
    }

    /// Device ID.
    pub fn device_id(&self) -> Option<u16> {
        if self.0.is_null() {
                None
        } else {
            let inner_obj = unsafe { *self.0 };
            Some(inner_obj.device_id)
        }
    }

    /// Domain (host bridge).
    pub fn domain(&self) -> Option<i32> {
        if self.0.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.0 };
            Some(inner_obj.domain)
        }
    }

    /// Bus inside domain.
    pub fn bus(&self) -> Option<u8> {
        if self.0.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.0 };
            Some(inner_obj.bus)
        }
    }

    /// Bus inside device.
    pub fn dev(&self) -> Option<u8> {
        if self.0.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.0 };
            Some(inner_obj.dev)
        }
    }

    /// Bus inside func.
    pub fn func(&self) -> Option<u8> {
        if self.0.is_null() {
            None
        } else {
            let inner_obj = unsafe { *self.0 };
            Some(inner_obj.func)
        }
    }

//     /// Vendor id and name of some compatible hardware.
//     ///
//     /// Note: Used mainly for ISA-PnP devices.
//     pub fn compat_vendor(&self) -> Option<HDID> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(HDID {
//                 id: inner_obj.compat_vendor.id,
//                 name: unsafe { c_char_to_str(inner_obj.compat_vendor.name) },
//             })
//         }
//     }

//     /// Device id and name of some compatible hardware.
//     ///
//     /// Note: Used mainly for ISA-PnP devices.
//     pub fn compat_device(&self) -> Option<HDID> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(HDID {
//                 id: inner_obj.compat_device.id,
//                 name: unsafe { c_char_to_str(inner_obj.compat_device.name) },
//             })
//         }
//     }

//     /// Hardware class.
//     ///
//     /// Note: Not to confuse with [`crate::HD::base_class`]!
//     pub fn hw_class(&self) -> Option<u32> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(inner_obj.hw_class)
//         }
//     }

//     /// Model name.
//     ///
//     /// Note: This is a combination of vendor and device names. Some heuristics is used
//     /// to make it more presentable. Use this instead of [`crate::HD::vendor`] and
//     /// [`crate::HD::device`].
//     pub fn model(&self) -> Option<&'static str> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(unsafe { c_char_to_str(inner_obj.model) })
//         }
//     }

//     /// Device this hardware is attached to.
//     ///
//     /// Note: Link to some 'parent' %device. Use \ref hd_get_device_by_idx() to get
//     /// the corresponding hardware entry.
//     pub fn attached_to(&self) -> Option<u32> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(inner_obj.attached_to)
//         }
//     }

//     /// sysfs entry for this hardware, if any.
//     pub fn sysfs_id(&self) -> Option<&'static str> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(unsafe { c_char_to_str(inner_obj.sysfs_id) })
//         }
//     }

//     /// sysfs bus id for this hardware, if any.
//     pub fn sysfs_bus_id(&self) -> Option<&'static str> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(unsafe { c_char_to_str(inner_obj.sysfs_bus_id) })
//         }
//     }

//     /// sysfs device link.
//     pub fn sysfs_device_link(&self) -> Option<&'static str> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(unsafe { c_char_to_str(inner_obj.sysfs_device_link) })
//         }
//     }

//     /// Special %device file.
//     ///
//     /// Note: Device file name to access this hardware.
//     pub fn unix_dev_name(&self) -> Option<&'static str> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(unsafe { c_char_to_str(inner_obj.unix_dev_name) })
//         }
//     }

//     /// Special %device file.
//     ///
//     /// Note: Device file name to access this hardware.
//     pub fn unix_dev_name2(&self) -> Option<&'static str> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(unsafe { c_char_to_str(inner_obj.unix_dev_name2) })
//         }
//     }

//     /// BIOS/PROM id.
//     ///
//     /// Note: Where appropriate, this is a special BIOS/PROM id (e.g. "0x80" for
//     /// the first harddisk on Intel-PCs).
//     /// CHPID for s390.
//     pub fn rom_id(&self) -> Option<&'static str> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(unsafe { c_char_to_str(inner_obj.rom_id) })
//         }
//     }

//     /// HAL udi.
//     pub fn udi(&self) -> Option<&'static str> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(unsafe { c_char_to_str(inner_obj.udi) })
//         }
//     }

//     /// [`crate::HD::udi`] of parent ([`crate::HD::attached_to`])
//     pub fn parent_udi(&self) -> Option<&'static str> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(unsafe { c_char_to_str(inner_obj.parent_udi) })
//         }
//     }

//     /// Unique id for this hardware.
//     ///
//     /// Note: A unique string identifying this hardware. The string consists
//     /// of two parts separated by a dot ("."). The part before the dot
//     /// describes the location (where the hardware is attached in the system).
//     /// The part after the dot identifies the hardware itself. The string
//     /// must not contain slashes ("/") because we're going to create files
//     /// with this id as name. Apart from this there are no restrictions on
//     /// the form of this string.
//     pub fn unique_id(&self) -> Option<&'static str> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(unsafe { c_char_to_str(inner_obj.unique_id) })
//         }
//     }

//     /// Currently active driver.
//     pub fn driver(&self) -> Option<&'static str> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(unsafe { c_char_to_str(inner_obj.driver) })
//         }
//     }

//     /// Currently active driver module (if any).
//     pub fn driver_module(&self) -> Option<&'static str> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(unsafe { c_char_to_str(inner_obj.driver_module) })
//         }
//     }

//     /// [`crate::HD::unique_id`] of parent ([`crate::HD::attached_to`])
//     pub fn parent_id(&self) -> Option<&'static str> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(unsafe { c_char_to_str(inner_obj.parent_id) })
//         }
//     }

//     /// USB Global Unique Identifier.
//     ///
//     /// Note: Available for USB devices. This may even be set if [`crate::HD::bus`] is not
//     /// bus_usb (e.g. USB storage devices will have [`crate::HD::bus`] set to
//     /// bus_scsi due to SCSI emulation).
//     pub fn usb_guid(&self) -> Option<&'static str> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(unsafe { c_char_to_str(inner_obj.usb_guid) })
//         }
//     }

//     /// module alias
//     pub fn modalias(&self) -> Option<&'static str> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(unsafe { c_char_to_str(inner_obj.modalias) })
//         }
//     }

//     /// Consistent Device Name (CDN), pci firmware spec 3.1, chapter 4.6.7
//     pub fn label(&self) -> Option<&'static str> {
//         if self.handle.is_null() {
//             None
//         } else {
//             let inner_obj = unsafe { *self.handle };
//             Some(unsafe { c_char_to_str(inner_obj.label) })
//         }
//     }

//     pub fn list(
//         data: &mut HDData,
//         item: HWItem,
//         rescan: i32,
//         hd_old: Option<&mut Self>,
//     ) -> Option<Self> {
//         let hd_old_cov = if hd_old.is_none() { ptr::null_mut() } else { hd_old.unwrap().handle };
//         Some(HD {
//             handle: unsafe { libhd_sys::hd_list(data.0, item as u32, rescan, hd_old_cov) },
//             is_list: true,
//             do_drop: false,
//             _phantom: PhantomData,
//         })
//     }

//     pub fn dump_entry(data: &mut HDData, hd: &mut Self) {
//         unsafe { libhd_sys::hd_dump_entry(data.0, hd.handle, libhd_sys::get_stdout_ptr()) }
//     }

//     pub fn len(hd: &Self) -> usize {
//         unsafe { libhd_sys::hd_get_len(hd.handle) }
//     }

//     pub fn iter(&self) -> Iter<'_> {
//         Iter { ptr: self.handle as _, _phantom: PhantomData }
//     }

     pub fn iter_mut(&self) -> IterMut<'_> {
         IterMut { ptr: self.0 as _, _phantom: PhantomData }
     }
}

// pub struct Iter<'a> {
//     ptr: *const libhd_sys::hd_t,
//     _phantom: PhantomData<&'a ()>,
// }

pub struct IterMut<'a> {
    ptr: *mut libpci_sys::pci_dev,
    _phantom: PhantomData<&'a mut ()>,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = PCIDevice<'a>;

    // next() is the only required method
    fn next(&mut self) -> Option<PCIDevice<'a>> {
        let ptr = self.ptr;
        self.ptr = unsafe { libpci_sys::pci_get_next_device_mut(self.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { PCIDevice::from_raw(ptr) })
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
        str::from_utf8(slice).expect("bad error message from libpci")
    }

    #[cfg(feature = "std")]
    {
        std::ffi::CStr::from_ptr(text).to_str().expect("bad error message from libpci")
    }
}
