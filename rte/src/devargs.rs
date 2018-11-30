use std::mem;
use std::os::unix::io::AsRawFd;

use cfile;

use ffi;

use errors::Result;

/// Type of generic device
#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum DevType {
    WhiteListed = ffi::rte_devtype::RTE_DEVTYPE_WHITELISTED_PCI,
    BlackListed = ffi::rte_devtype::RTE_DEVTYPE_BLACKLISTED_PCI,
    Virtual = ffi::rte_devtype::RTE_DEVTYPE_VIRTUAL,
}

impl From<DevType> for ffi::rte_devtype::Type {
    fn from(v: DevType) -> Self {
        unsafe { mem::transmute(v) }
    }
}

/// Add a device to the user device list
pub fn add(devtype: DevType, devargs: &str) -> Result<()> {
    rte_check!(unsafe { ffi::rte_devargs_add(devtype.into(), try!(to_cptr!(devargs))) })
}

/// Count the number of user devices of a specified type
pub fn type_count(devtype: DevType) -> usize {
    unsafe { ffi::rte_devargs_type_count(devtype.into()) as usize }
}

/// This function dumps the list of user device and their arguments.
pub fn dump<S: AsRawFd>(s: &S) {
    if let Ok(f) = cfile::open_stream(s, "w") {
        unsafe {
            ffi::rte_devargs_dump(f.stream() as *mut ffi::FILE);
        }
    }
}
