use ffi::*;

pub trait Device {}

pub struct RawDevice(u8);

/// Get the total number of Ethernet devices that have been successfully initialized
/// by the matching Ethernet driver during the PCI probing phase.
///
/// All devices whose port identifier is in the range [0, rte::ethdev::count() - 1]
/// can be operated on by network applications immediately after invoking rte_eal_init().
/// If the application unplugs a port using hotplug function, The enabled port numbers may be noncontiguous.
/// In the case, the applications need to manage enabled port by themselves.
pub fn count() -> usize {
    unsafe { rte_eth_dev_count() as usize }
}
