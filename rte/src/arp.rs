use ffi;

pub use ffi::{
    RTE_ARP_HRD_ETHER, RTE_ARP_OP_INVREPLY, RTE_ARP_OP_INVREQUEST, RTE_ARP_OP_REPLY, RTE_ARP_OP_REQUEST, RTE_ARP_OP_REVREPLY, RTE_ARP_OP_REVREQUEST,
};

/// ARP header IPv4 payload.
pub type ArpIpv4 = ffi::rte_arp_ipv4;

/// ARP header.
pub type ArpHdr = ffi::rte_arp_hdr;
