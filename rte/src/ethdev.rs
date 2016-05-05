use std::fmt;
use std::ffi::CStr;

use ffi;

/// Retrieve the contextual information of an Ethernet device.
pub fn dev_info(port_id: u8) -> DeviceInfo {
    let mut info: Box<ffi::Struct_rte_eth_dev_info> = Box::new(Default::default());

    unsafe { ffi::rte_eth_dev_info_get(port_id, info.as_mut()) }

    DeviceInfo(info)
}

pub struct DeviceInfo(Box<ffi::Struct_rte_eth_dev_info>);

impl fmt::Debug for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "DeviceInfo {{ driver_name: \"{}\", if_index: {} }}",
               self.driver_name(),
               self.if_index())
    }
}

impl DeviceInfo {
    /// Device Driver name.
    pub fn driver_name(&self) -> &str {
        unsafe { CStr::from_ptr((*self.0).driver_name).to_str().unwrap() }
    }

    /// Index to bound host interface, or 0 if none. Use if_indextoname() to translate into an interface name.
    pub fn if_index(&self) -> u32 {
        (*self.0).if_index
    }
}

/// Get the total number of Ethernet devices that have been successfully initialized
/// by the matching Ethernet driver during the PCI probing phase.
///
/// All devices whose port identifier is in the range [0, rte::ethdev::count() - 1]
/// can be operated on by network applications immediately after invoking rte_eal_init().
/// If the application unplugs a port using hotplug function, The enabled port numbers may be noncontiguous.
/// In the case, the applications need to manage enabled port by themselves.
pub fn count() -> u32 {
    unsafe { ffi::rte_eth_dev_count() as u32 }
}
