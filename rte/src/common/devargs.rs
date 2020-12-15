use std::os::unix::io::AsRawFd;

use cfile;

use ffi;

use errors::Result;

/// Type of generic device
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive, ToPrimitive)]
pub enum DevType {
    Allowed = ffi::rte_devtype::RTE_DEVTYPE_ALLOWED,
    Blocked = ffi::rte_devtype::RTE_DEVTYPE_BLOCKED,
    Virtual = ffi::rte_devtype::RTE_DEVTYPE_VIRTUAL,
}

/// Add a device to the user device list
pub fn add(devtype: DevType, devargs: &str) -> Result<()> {
    rte_check!(unsafe { ffi::rte_devargs_add(devtype as u32, try!(to_cptr!(devargs))) })
}

/// Count the number of user devices of a specified type
pub fn type_count(devtype: DevType) -> usize {
    unsafe { ffi::rte_devargs_type_count(devtype as u32) as usize }
}

/// This function dumps the list of user device and their arguments.
pub fn dump<S: AsRawFd>(s: &S) {
    if let Ok(f) = cfile::fdopen(s, "w") {
        unsafe {
            ffi::rte_devargs_dump(f.stream() as *mut ffi::FILE);
        }
    }
}
