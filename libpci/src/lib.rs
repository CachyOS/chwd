#![no_std]

#[cfg(feature = "std")]
extern crate std;

// Re-export libpci-c-sys
pub use libpci_c_sys;

#[cfg(feature = "std")]
use std::os::raw::c_char;

#[cfg(not(feature = "std"))]
use libc::c_char;

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

use alloc::string::String;
use core::marker::PhantomData;
use core::{mem, ptr, str};

/// Returns the LIBPCI version.
pub fn version_number() -> u32 {
    libpci_c_sys::PCI_LIB_VERSION as u32
}

#[repr(u32)]
#[derive(Clone, PartialEq)]
pub enum Fill {
    IDENT = libpci_c_sys::PCI_FILL_IDENT as u32,
    IRQ = libpci_c_sys::PCI_FILL_IRQ as u32,
    BASES = libpci_c_sys::PCI_FILL_BASES as u32,
    RomBase = libpci_c_sys::PCI_FILL_ROM_BASE as u32,
    SIZES = libpci_c_sys::PCI_FILL_SIZES as u32,
    CLASS = libpci_c_sys::PCI_FILL_CLASS as u32,
    CAPS = libpci_c_sys::PCI_FILL_CAPS as u32,
    ExtCaps = libpci_c_sys::PCI_FILL_EXT_CAPS as u32,
    PhysSlot = libpci_c_sys::PCI_FILL_PHYS_SLOT as u32,
    ModuleAlias = libpci_c_sys::PCI_FILL_MODULE_ALIAS as u32,
    LABEL = libpci_c_sys::PCI_FILL_LABEL as u32,
    NumaNode = libpci_c_sys::PCI_FILL_NUMA_NODE as u32,
    IoFlags = libpci_c_sys::PCI_FILL_IO_FLAGS as u32,
    DtNode = libpci_c_sys::PCI_FILL_DT_NODE as u32,
    IommuGroup = libpci_c_sys::PCI_FILL_IOMMU_GROUP as u32,
    BridgeBases = libpci_c_sys::PCI_FILL_BRIDGE_BASES as u32,
    RESCAN = libpci_c_sys::PCI_FILL_RESCAN as u32,
    ClassExt = libpci_c_sys::PCI_FILL_CLASS_EXT as u32,
    SUBSYS = libpci_c_sys::PCI_FILL_SUBSYS as u32,
    PARENT = libpci_c_sys::PCI_FILL_PARENT as u32,
    DRIVER = libpci_c_sys::PCI_FILL_DRIVER as u32,
}

#[repr(u32)]
#[derive(Clone, PartialEq)]
pub enum AccessType {
    Auto = libpci_c_sys::pci_access_type_PCI_ACCESS_AUTO,
    SysBusPci = libpci_c_sys::pci_access_type_PCI_ACCESS_SYS_BUS_PCI,
    ProcBusPci = libpci_c_sys::pci_access_type_PCI_ACCESS_PROC_BUS_PCI,
    I386Type1 = libpci_c_sys::pci_access_type_PCI_ACCESS_I386_TYPE1,
    I386Type2 = libpci_c_sys::pci_access_type_PCI_ACCESS_I386_TYPE2,
    FbsdDevice = libpci_c_sys::pci_access_type_PCI_ACCESS_FBSD_DEVICE,
    AixDevice = libpci_c_sys::pci_access_type_PCI_ACCESS_AIX_DEVICE,
    NbsdLibpci = libpci_c_sys::pci_access_type_PCI_ACCESS_NBSD_LIBPCI,
    ObsdDevice = libpci_c_sys::pci_access_type_PCI_ACCESS_OBSD_DEVICE,
    Dump = libpci_c_sys::pci_access_type_PCI_ACCESS_DUMP,
    Darwin = libpci_c_sys::pci_access_type_PCI_ACCESS_DARWIN,
    SylixosDevice = libpci_c_sys::pci_access_type_PCI_ACCESS_SYLIXOS_DEVICE,
    HURD = libpci_c_sys::pci_access_type_PCI_ACCESS_HURD,
    Win32Cfgmgr32 = libpci_c_sys::pci_access_type_PCI_ACCESS_WIN32_CFGMGR32,
    Win32Kldbg = libpci_c_sys::pci_access_type_PCI_ACCESS_WIN32_KLDBG,
    Win32Sysdbg = libpci_c_sys::pci_access_type_PCI_ACCESS_WIN32_SYSDBG,
    MmioType1 = libpci_c_sys::pci_access_type_PCI_ACCESS_MMIO_TYPE1,
    Type1Ext = libpci_c_sys::pci_access_type_PCI_ACCESS_MMIO_TYPE1_EXT,
    Max = libpci_c_sys::pci_access_type_PCI_ACCESS_MAX,
}

impl From<u32> for AccessType {
    fn from(value: u32) -> Self {
        if value > AccessType::Max as u32 || value < AccessType::Auto as u32 {
            return AccessType::Auto;
        }
        unsafe { mem::transmute(value) }
    }
}

/// PCI access structure.
#[derive(Clone, Debug)]
pub struct PCIAccess<'a> {
    handle: *mut libpci_c_sys::pci_access,
    _phantom: PhantomData<&'a ()>,
}

/// Holds device data found on this bus.
pub struct PCIDevice<'a>(*mut libpci_c_sys::pci_dev, PhantomData<&'a ()>);

unsafe impl<'a> Send for PCIAccess<'a> {}
unsafe impl<'a> Send for PCIDevice<'a> {}

impl<'a> Drop for PCIDevice<'a> {
    fn drop(&mut self) {}
}

impl<'a> Drop for PCIAccess<'a> {
    fn drop(&mut self) {
        if self.handle.is_null() {
            return;
        }
        // Safety: Just FFI
        unsafe {
            libpci_c_sys::pci_cleanup(self.handle);
        }
    }
}

impl<'a> PCIAccess<'a> {
    /// Tries to create a new data.
    ///
    /// Safety: Just FFI
    ///
    /// Returns `None` if libpci_c_sys::pci_alloc returns a NULL pointer - may happen if allocation
    /// fails.
    pub fn try_new(do_scan: bool) -> Option<Self> {
        let ptr: *mut libpci_c_sys::pci_access = unsafe { libpci_c_sys::pci_alloc() };
        if ptr.is_null() {
            None
        } else {
            unsafe {
                libpci_c_sys::pci_init(ptr);
            }
            if do_scan {
                unsafe {
                    libpci_c_sys::pci_scan_bus(ptr);
                }
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
            unsafe {
                libpci_c_sys::pci_scan_bus(self.handle);
            }
        }
    }

    /// Get linked list of devices
    pub fn devices(&mut self) -> Option<PCIDevice> {
        if self.handle.is_null() {
            None
        } else {
            Some(unsafe { PCIDevice::from_raw((*self.handle).devices) })
        }
    }
}

impl<'a> Default for PCIDevice<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> PCIDevice<'a> {
    /// Create a new data.
    pub fn new() -> Self {
        // Safety: Just FFI
        Self(ptr::null_mut::<libpci_c_sys::pci_dev>(), PhantomData)
    }

    /// Constructs from raw C type
    ///
    /// # Safety
    ///
    /// the caller must guarantee that `self` is valid
    /// for a reference if it isn't null.
    pub unsafe fn from_raw(data: *mut libpci_c_sys::pci_dev) -> Self {
        Self(data, PhantomData)
    }

    /// Scan to get the list of devices
    pub fn fill_info(&mut self, fill: u32) {
        if !self.0.is_null() {
            unsafe {
                libpci_c_sys::pci_fill_info(self.0, fill as _);
            }
        }
    }

    /// Get class str.
    pub fn class(&self) -> Option<String> {
        if self.0.is_null() {
            None
        } else {
            let mut class = vec![0_u8; 1024];
            let size = (class.len() * mem::size_of::<u8>()) as usize;

            unsafe {
                libpci_c_sys::pci_lookup_class_helper(
                    (*self.0).access,
                    class.as_mut_ptr() as _,
                    size,
                    self.0,
                );
            }
            Some(String::from(unsafe { c_char_to_str(class.as_ptr() as _) }))
        }
    }

    /// Get vendor str.
    pub fn vendor(&self) -> Option<String> {
        if self.0.is_null() {
            None
        } else {
            let mut vendor = vec![0_u8; 256];
            let size = (vendor.len() * mem::size_of::<u8>()) as usize;

            unsafe {
                libpci_c_sys::pci_lookup_vendor_helper(
                    (*self.0).access,
                    vendor.as_mut_ptr() as _,
                    size,
                    self.0,
                );
            }
            Some(String::from(unsafe { c_char_to_str(vendor.as_ptr() as _) }))
        }
    }

    /// Get device str.
    pub fn device(&self) -> Option<String> {
        if self.0.is_null() {
            None
        } else {
            let mut device = vec![0_u8; 256];
            let size = (device.len() * mem::size_of::<u8>()) as usize;

            unsafe {
                libpci_c_sys::pci_lookup_device_helper(
                    (*self.0).access,
                    device.as_mut_ptr() as _,
                    size,
                    self.0,
                );
            }
            Some(String::from(unsafe { c_char_to_str(device.as_ptr() as _) }))
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

    pub fn iter_mut(&self) -> IterMut<'_> {
        IterMut { ptr: self.0 as _, _phantom: PhantomData }
    }
}

pub struct IterMut<'a> {
    ptr: *mut libpci_c_sys::pci_dev,
    _phantom: PhantomData<&'a mut ()>,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = PCIDevice<'a>;

    // next() is the only required method
    fn next(&mut self) -> Option<PCIDevice<'a>> {
        let ptr = self.ptr;
        self.ptr = unsafe { libpci_c_sys::pci_get_next_device_mut(self.ptr) };
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
