use ffi;

pub use ffi::{ARP_HRD_ETHER, ARP_OP_REQUEST, ARP_OP_REPLY};

/// ARP header IPv4 payload.
pub type ArpIpv4 = ffi::arp_ipv4;

/// ARP header.
pub type ArpHdr = ffi::arp_hdr;
