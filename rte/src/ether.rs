use std::fmt;
use std::str;
use std::ptr;
use std::error;
use std::result;
use std::ops::{Deref, DerefMut};

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

const ETHER_ADDR_LEN: usize = 6;

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
    pub fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> EtherAddr {
        EtherAddr([a, b, c, d, e, f])
    }

    pub fn copy(from: &[u8; ETHER_ADDR_LEN], to: &mut [u8; ETHER_ADDR_LEN]) {
        unsafe { ptr::copy_nonoverlapping(from.as_ptr(), to.as_mut_ptr(), ETHER_ADDR_LEN) }
    }

    /// Returns the six eight-bit integers that make up this address.
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_macaddr() {
        let addr = EtherAddr::new(0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f);

        assert_eq!(addr.octets(), &[0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f]);
        assert_eq!(addr.to_string(), "1a:2b:3c:4d:5e:6f");

        assert_eq!(addr, EtherAddr::from([0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f]));
        assert_eq!(addr, EtherAddr::from_str("1a:2b:3c:4d:5e:6f").unwrap());
    }
}
