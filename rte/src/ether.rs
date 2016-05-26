use std::fmt;
use std::str;
use std::mem;
use std::ptr;
use std::error;
use std::result;
use std::ops::{Deref, DerefMut};

use rand::{Rng, thread_rng};

use ffi;

#[derive(Debug, Clone, PartialEq)]
pub struct AddrParseError(());

impl fmt::Display for AddrParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(error::Error::description(self))
    }
}

impl error::Error for AddrParseError {
    fn description(&self) -> &str {
        "invalid MAC address syntax"
    }
}

pub const ETHER_ADDR_LEN: usize = 6;

/// A 48-bit (6 byte) buffer containing the MAC address
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct EtherAddr([u8; ETHER_ADDR_LEN]);

impl Deref for EtherAddr {
    type Target = [u8; ETHER_ADDR_LEN];

    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

impl DerefMut for EtherAddr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.0;
    }
}

impl EtherAddr {
    /// Creates a new MAC address from six eight-bit octets.
    ///
    /// The result will represent the MAC address a:b:c:d:e:f.
    #[inline]
    pub fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> EtherAddr {
        EtherAddr([a, b, c, d, e, f])
    }

    /// Fast copy an Ethernet address.
    #[inline]
    pub fn copy(from: &[u8; ETHER_ADDR_LEN], to: &mut [u8; ETHER_ADDR_LEN]) {
        unsafe { ptr::copy_nonoverlapping(from.as_ptr(), to.as_mut_ptr(), ETHER_ADDR_LEN) }
    }

    /// Returns the six eight-bit integers that make up this address.
    #[inline]
    pub fn octets(&self) -> &[u8; ETHER_ADDR_LEN] {
        &self.0
    }

    pub fn from_bytes(b: &[u8]) -> result::Result<Self, AddrParseError> {
        if b.len() != ETHER_ADDR_LEN {
            return Err(AddrParseError(()));
        }

        let mut addr = [0; ETHER_ADDR_LEN];

        unsafe {
            ptr::copy(b.as_ptr(), addr.as_mut().as_mut_ptr(), b.len());
        }

        Ok(EtherAddr(addr))
    }

    pub fn zeroed() -> Self {
        unsafe { mem::zeroed() }
    }

    pub fn broadcast() -> Self {
        EtherAddr([0xffu8; ETHER_ADDR_LEN])
    }

    /// Generate a random Ethernet address that is locally administered and not multicast.
    pub fn random() -> Self {
        let mut addr = [0u8; ETHER_ADDR_LEN];

        thread_rng().fill_bytes(&mut addr);

        addr[0] &= !ffi::ETHER_GROUP_ADDR as u8; // clear multicast bit
        addr[0] |= ffi::ETHER_LOCAL_ADMIN_ADDR as u8; // set local assignment bit

        EtherAddr(addr)
    }

    /// Check if an Ethernet address is filled with zeros.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0 == Self::zeroed().0
    }

    /// Check if an Ethernet address is a unicast address.
    #[inline]
    pub fn is_unicast(&self) -> bool {
        (self.0[0] & ffi::ETHER_GROUP_ADDR as u8) == 0
    }

    /// Check if an Ethernet address is a multicast address.
    #[inline]
    pub fn is_multicast(&self) -> bool {
        (self.0[0] & ffi::ETHER_GROUP_ADDR as u8) != 0
    }

    /// Check if an Ethernet address is a broadcast address.
    #[inline]
    pub fn is_broadcast(&self) -> bool {
        self.0 == Self::broadcast().0
    }

    /// Check if an Ethernet address is a universally assigned address.
    #[inline]
    pub fn is_universal(&self) -> bool {
        (self.0[0] & ffi::ETHER_LOCAL_ADMIN_ADDR as u8) == 0
    }

    ///  Check if an Ethernet address is a locally assigned address.
    #[inline]
    pub fn is_local_admin(&self) -> bool {
        (self.0[0] & ffi::ETHER_LOCAL_ADMIN_ADDR as u8) != 0
    }

    /// Check if an Ethernet address is a valid address.
    ///
    /// Checks that the address is a unicast address and is not filled with zeros.
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.is_unicast() && !self.is_zero()
    }
}

impl fmt::Display for EtherAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
               self.0[0],
               self.0[1],
               self.0[2],
               self.0[3],
               self.0[4],
               self.0[5])
    }
}

impl From<[u8; 6]> for EtherAddr {
    fn from(addr: [u8; 6]) -> EtherAddr {
        EtherAddr(addr)
    }
}

impl str::FromStr for EtherAddr {
    type Err = AddrParseError;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let addr: Vec<u8> = s.split(':')
            .filter_map(|part| u8::from_str_radix(part, 16).ok())
            .collect();

        EtherAddr::from_bytes(addr.as_slice())
    }
}

/// Ethernet header: Contains the destination address, source address and frame type.
#[repr(C)]
pub struct EtherHdr {
    /// Destination address.
    pub d_addr: [u8; 6],
    /// Source address.
    pub s_addr: [u8; 6],
    /// Frame type.
    pub ether_type: u16,
}

/// Ethernet VLAN Header.
///
/// Contains the 16-bit VLAN Tag Control Identifier
/// and the Ethernet type of the encapsulated frame.
///
pub struct VlanHdr {
    /// Priority (3) + CFI (1) + Identifier Code (12)
    pub vlan_tci: u16,
    /// Ethernet type of encapsulated frame.
    pub eth_proto: u16,
}

/// VXLAN protocol header.
///
/// Contains the 8-bit flag, 24-bit VXLAN Network Identifier
/// and Reserved fields (24 bits and 8 bits)
///
pub struct VxlanHdr {
    /// flag (8) + Reserved (24).
    pub vx_flags: u32,
    /// VNI (24) + Reserved (8).
    pub vx_vni: u32,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_macaddr() {
        let addr = EtherAddr::new(0x18, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f);

        assert_eq!(addr.octets(), &[0x18, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f]);
        assert_eq!(addr.to_string(), "18:2b:3c:4d:5e:6f");

        assert_eq!(addr, EtherAddr::from([0x18, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f]));
        assert_eq!(addr, EtherAddr::from_str("18:2b:3c:4d:5e:6f").unwrap());

        assert!(!addr.is_zero());
        assert!(EtherAddr::zeroed().is_zero());

        assert!(addr.is_unicast());

        let local_addr = EtherAddr::new(0x13, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f);

        assert!(!addr.is_multicast());
        assert!(local_addr.is_multicast());

        assert!(!addr.is_broadcast());
        assert!(EtherAddr::broadcast().is_broadcast());

        assert!(addr.is_universal());
        assert!(!local_addr.is_universal());

        assert!(!addr.is_local_admin());
        assert!(local_addr.is_local_admin());

        let rand_addr = EtherAddr::random();

        assert!(rand_addr.is_unicast());
        assert!(!rand_addr.is_multicast());
        assert!(!rand_addr.is_broadcast());
        assert!(!rand_addr.is_universal());
        assert!(rand_addr.is_local_admin());
        assert!(rand_addr.is_valid());
    }
}
