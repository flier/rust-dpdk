use std::mem;
use std::ops::{Deref, DerefMut};

use ffi;

use errors::Result;
use memory::SocketId;
use ethdev;
use ether;

/// Supported modes of operation of link bonding library
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum BondMode {
    /// Round Robin (Mode 0).
    ///
    /// In this mode all transmitted packets will be balanced equally across all
    /// active slaves of the bonded in a round robin fashion.
    ///
    RouncRobin = ffi::BONDING_MODE_ROUND_ROBIN as u8,

    /// Active Backup (Mode 1).
    ///
    /// In this mode all packets transmitted will be transmitted on the primary
    /// slave until such point as the primary slave is no longer available and then
    /// transmitted packets will be sent on the next available slaves. The primary
    /// slave can be defined by the user but defaults to the first active slave
    /// available if not specified.
    ///
    ActiveBackup = ffi::BONDING_MODE_ACTIVE_BACKUP as u8,

    /// Balance (Mode 2).
    ///
    /// In this mode all packets transmitted will be balanced across the available
    /// slaves using one of three available transmit policies - l2, l2+3 or l3+4.
    /// See BALANCE_XMIT_POLICY macros definitions for further details on transmit
    /// policies.
    ///
    Balance = ffi::BONDING_MODE_BALANCE as u8,

    /// Broadcast (Mode 3).
    ///
    /// In this mode all transmitted packets will be transmitted on all available
    /// active slaves of the bonded.
    ///
    Broadcast = ffi::BONDING_MODE_BROADCAST as u8,

    /// 802.3AD (Mode 4).
    ///
    /// This mode provides auto negotiation/configuration
    /// of peers and well as link status changes monitoring using out of band
    /// LACP (link aggregation control protocol) messages. For further details of
    /// LACP specification see the IEEE 802.3ad/802.1AX standards. It is also
    /// described here
    /// https://www.kernel.org/doc/Documentation/networking/bonding.txt.
    ///
    /// Important Usage Notes:
    /// - for LACP mode to work the rx/tx burst functions must be invoked
    /// at least once every 100ms, otherwise the out-of-band LACP messages will not
    /// be handled with the expected latency and this may cause the link status to be
    /// incorrectly marked as down or failure to correctly negotiate with peers.
    /// - For optimal performance during initial handshaking the array of mbufs provided
    /// to rx_burst should be at least 2 times the slave count size.
    ///
    AutoNeg = ffi::BONDING_MODE_8023AD as u8,

    /// Adaptive TLB (Mode 5)
    ///
    /// This mode provides an adaptive transmit load balancing. It dynamically
    /// changes the transmitting slave, according to the computed load. Statistics
    /// are collected in 100ms intervals and scheduled every 10ms
    ///
    AdaptiveTLB = ffi::BONDING_MODE_TLB as u8,

    /// Adaptive Load Balancing (Mode 6)
    ///
    /// This mode includes adaptive TLB and receive load balancing (RLB). In RLB the
    /// bonding driver intercepts ARP replies send by local system and overwrites its
    /// source MAC address, so that different peers send data to the server on
    /// different slave interfaces. When local system sends ARP request, it saves IP
    /// information from it. When ARP reply from that peer is received, its MAC is
    /// stored, one of slave MACs assigned and ARP reply send to that peer.
    ///
    AdaptiveLB = ffi::BONDING_MODE_ALB as u8,
}

impl From<u8> for BondMode {
    fn from(v: u8) -> Self {
        unsafe { mem::transmute(v) }
    }
}

/// Balance Mode Transmit Policies
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TransmitPolicy {
    /// Layer 2 (Ethernet MAC)
    Layer2,
    /// Layer 2+3 (Ethernet MAC + IP Addresses) transmit load balancing
    Layer23,
    /// Layer 3+4 (IP Addresses + UDP Ports) transmit load balancing
    Layer34,
}

impl From<u8> for TransmitPolicy {
    fn from(v: u8) -> Self {
        unsafe { mem::transmute(v) }
    }
}

pub struct BondedDevice(ethdev::EthDevice);

impl Deref for BondedDevice {
    type Target = ethdev::EthDevice;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BondedDevice {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Create a bonded rte_eth_dev device
pub fn create(name: &str, mode: BondMode, socket_id: SocketId) -> Result<BondedDevice> {
    let port_id = unsafe { ffi::rte_eth_bond_create(cstr!(name), mode as u8, socket_id as u8) };

    rte_check!(port_id; ok => { dev(port_id as ethdev::PortId) })
}

/// Free a bonded rte_eth_dev device
pub fn free(name: &str) -> Result<()> {
    rte_check!(unsafe { ffi::rte_eth_bond_free(cstr!(name)) })
}

pub fn dev(port_id: ethdev::PortId) -> BondedDevice {
    BondedDevice(ethdev::dev(port_id))
}

impl BondedDevice {
    /// Add a rte_eth_dev device as a slave to the bonded device
    pub fn add_slave(&self, slave: &ethdev::EthDevice) -> Result<&Self> {
        rte_check!(unsafe {
            ffi::rte_eth_bond_slave_add(self.0.portid(), slave.portid())
        }; ok => { self })
    }

    /// Remove a slave rte_eth_dev device from the bonded device
    pub fn remove_slave(&self, slave: &ethdev::EthDevice) -> Result<&Self> {
        rte_check!(unsafe {
            ffi::rte_eth_bond_slave_remove(self.0.portid(), slave.portid())
        }; ok => { self })
    }

    /// Get link bonding mode of bonded device
    pub fn mode(&self) -> Result<BondMode> {
        let mode = unsafe { ffi::rte_eth_bond_mode_get(self.0.portid()) };

        rte_check!(mode; ok => { BondMode::from(mode as u8) })
    }

    /// Set link bonding mode of bonded device
    pub fn set_mode(&self, mode: BondMode) -> Result<&Self> {
        rte_check!(unsafe {
            ffi::rte_eth_bond_mode_set(self.0.portid(), mode as u8)
        }; ok => { self })
    }

    /// Get primary slave of bonded device
    pub fn primary(&self) -> Result<ethdev::EthDevice> {
        let portid = unsafe { ffi::rte_eth_bond_primary_get(self.0.portid()) };

        rte_check!(portid; ok => { ethdev::dev(portid as u8) })
    }

    /// Set slave rte_eth_dev as primary slave of bonded device
    pub fn set_primary(&self, dev: &ethdev::EthDevice) -> Result<&Self> {
        rte_check!(unsafe {
            ffi::rte_eth_bond_primary_set(self.0.portid(), dev.portid())
        }; ok => { self })
    }

    /// Populate an array with list of the slaves port id's of the bonded device
    pub fn slaves(&self) -> Result<Vec<ethdev::EthDevice>> {
        let mut slaves = [0u8; ffi::RTE_MAX_ETHPORTS as usize];

        let num = unsafe {
            ffi::rte_eth_bond_slaves_get(self.0.portid(), slaves.as_mut_ptr(), slaves.len() as u8)
        };

        rte_check!(num; ok => {
            slaves[..num as usize].iter().map(|portid| ethdev::dev(*portid)).collect()
        })
    }

    /// Populate an array with list of the active slaves port id's of the bonded device.
    pub fn active_slaves(&self) -> Result<Vec<ethdev::EthDevice>> {
        let mut slaves = [0u8; ffi::RTE_MAX_ETHPORTS as usize];

        let num = unsafe {
            ffi::rte_eth_bond_slaves_get(self.0.portid(), slaves.as_mut_ptr(), slaves.len() as u8)
        };

        rte_check!(num; ok => {
            slaves[..num as usize].iter().map(|portid| ethdev::dev(*portid)).collect()
        })
    }

    /// Set explicit MAC address to use on bonded device and it's slaves.
    pub fn set_mac_addr(&self, mac_addr: &ether::EtherAddr) -> Result<&Self> {
        rte_check!(unsafe {
            ffi::rte_eth_bond_mac_address_set(self.0.portid(),
                                              mem::transmute(mac_addr.octets().as_ptr()))
        }; ok => { self })
    }

    /// Reset bonded device to use MAC from primary slave on bonded device and it's slaves.
    pub fn reset_mac_addr(&self) -> Result<&Self> {
        rte_check!(unsafe {
            ffi::rte_eth_bond_mac_address_reset(self.0.portid())
        }; ok => { self })
    }

    /// Get the transmit policy set on bonded device for balance mode operation
    pub fn xmit_policy(&self) -> Result<TransmitPolicy> {
        let policy = unsafe { ffi::rte_eth_bond_xmit_policy_get(self.0.portid()) };

        rte_check!(policy; ok => { TransmitPolicy::from(policy as u8) })
    }

    /// Set the transmit policy for bonded device to use when it is operating in balance mode,
    /// this parameter is otherwise ignored in other modes of operation.
    pub fn set_xmit_policy(&self, policy: TransmitPolicy) -> Result<&Self> {
        rte_check!(unsafe {
            ffi::rte_eth_bond_xmit_policy_set(self.0.portid(), policy as u8)
        }; ok => { self })
    }
}
