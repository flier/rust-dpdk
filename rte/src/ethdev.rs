use std::fmt;
use std::ptr;
use std::mem;
use std::ffi::{CStr, CString};
use std::os::raw::c_void;

use libc;

use ffi;

use errors::{Error, Result};
use mempool;
use malloc;
use pci;
use ether::EtherAddr;

/// A structure used to retrieve link-level information of an Ethernet port.
pub struct EthLink {
    pub speed: u32,
    pub duplex: bool,
    pub autoneg: bool,
    pub up: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EthDevice(u8);

impl From<u8> for EthDevice {
    fn from(portid: u8) -> Self {
        EthDevice(portid)
    }
}

impl EthDevice {
    pub fn portid(&self) -> u8 {
        self.0
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

    /// Attach a new Ethernet device specified by aruguments.
    pub fn attach(devargs: &str) -> Result<Self> {
        let mut portid: u8 = 0;

        let ret = unsafe {
            ffi::rte_eth_dev_attach(try!(CString::new(devargs)).as_ptr(), &mut portid)
        };

        rte_check!(ret; ok => { EthDevice(portid) })
    }

    /// Configure an Ethernet device.
    ///
    /// This function must be invoked first before any other function in the Ethernet API.
    /// This function can also be re-invoked when a device is in the stopped state.
    ///
    pub fn configure(&self, nb_rx_queue: u16, nb_tx_queue: u16, conf: &EthConf) -> Result<()> {
        rte_check!(unsafe {
            ffi::rte_eth_dev_configure(self.0,
                                       nb_rx_queue,
                                       nb_tx_queue,
                                       RawEthConf::from(conf).as_raw())
        })
    }

    /// Retrieve the contextual information of an Ethernet device.
    pub fn info(&self) -> EthDeviceInfo {
        let mut info: Box<ffi::Struct_rte_eth_dev_info> = Box::new(Default::default());

        unsafe { ffi::rte_eth_dev_info_get(self.0, info.as_mut()) }

        EthDeviceInfo(info)
    }

    /// Retrieve the Ethernet address of an Ethernet device.
    pub fn macaddr(&self) -> EtherAddr {
        unsafe {
            let mut addr: ffi::Struct_ether_addr = mem::zeroed();

            ffi::rte_eth_macaddr_get(self.0, &mut addr);

            EtherAddr::from(addr.addr_bytes)
        }
    }

    /// Return the NUMA socket to which an Ethernet device is connected
    pub fn socket_id(&self) -> i32 {
        unsafe { ffi::rte_eth_dev_socket_id(self.0) }
    }

    /// Check if port_id of device is attached
    pub fn is_attached(&self) -> bool {
        unsafe { ffi::rte_eth_dev_is_valid_port(self.0) != 0 }
    }

    /// Allocate and set up a receive queue for an Ethernet device.
    ///
    /// The function allocates a contiguous block of memory for *nb_rx_desc*
    /// receive descriptors from a memory zone associated with *socket_id*
    /// and initializes each receive descriptor with a network buffer allocated
    /// from the memory pool *mb_pool*.
    pub fn rx_queue_setup(&self,
                          rx_queue_id: u16,
                          nb_rx_desc: u16,
                          rx_conf: Option<ffi::Struct_rte_eth_rxconf>,
                          mb_pool: &mempool::RawMemoryPool)
                          -> Result<()> {
        rte_check!(unsafe {
            ffi::rte_eth_rx_queue_setup(self.0,
                                        rx_queue_id,
                                        nb_rx_desc,
                                        self.socket_id() as u32,
                                        mem::transmute(&rx_conf),
                                        mb_pool.as_raw())
        })
    }

    /// Allocate and set up a transmit queue for an Ethernet device.
    pub fn tx_queue_setup(&self,
                          tx_queue_id: u16,
                          nb_tx_desc: u16,
                          tx_conf: Option<ffi::Struct_rte_eth_txconf>)
                          -> Result<()> {
        rte_check!(unsafe {
            ffi::rte_eth_tx_queue_setup(self.0,
                                        tx_queue_id,
                                        nb_tx_desc,
                                        self.socket_id() as u32,
                                        mem::transmute(&tx_conf))
        })
    }

    /// Enable receipt in promiscuous mode for an Ethernet device.
    pub fn promiscuous_enable(&self) {
        unsafe { ffi::rte_eth_promiscuous_enable(self.0) }
    }

    /// Disable receipt in promiscuous mode for an Ethernet device.
    pub fn promiscuous_disable(&self) {
        unsafe { ffi::rte_eth_promiscuous_disable(self.0) }
    }

    /// Return the value of promiscuous mode for an Ethernet device.
    pub fn is_promiscuous_enabled(&self) -> Result<bool> {
        let ret = unsafe { ffi::rte_eth_promiscuous_get(self.0) };

        rte_check!(ret; ok => { ret != 0 })
    }

    /// Retrieve the status (ON/OFF), the speed (in Mbps) and
    /// the mode (HALF-DUPLEX or FULL-DUPLEX) of the physical link of an Ethernet device.
    ///
    /// It might need to wait up to 9 seconds in it.
    ///
    pub fn link(&self) -> EthLink {
        let link = 0u64;

        unsafe { ffi::rte_eth_link_get(self.0, mem::transmute(&link)) }

        EthLink {
            speed: (link & 0xFFFFFFFF) as u32,
            duplex: (link & (1 << 32)) != 0,
            autoneg: (link & (1 << 33)) != 0,
            up: (link & (1 << 34)) != 0,
        }
    }

    /// Retrieve the status (ON/OFF), the speed (in Mbps) and
    /// the mode (HALF-DUPLEX or FULL-DUPLEX) of the physical link of an Ethernet device.
    ///
    /// It is a no-wait version of rte_eth_link_get().
    ///
    pub fn link_nowait(&self) -> EthLink {
        let link = 0u64;

        unsafe { ffi::rte_eth_link_get_nowait(self.0, mem::transmute(&link)) }

        EthLink {
            speed: (link & 0xFFFFFFFF) as u32,
            duplex: (link & (1 << 32)) != 0,
            autoneg: (link & (1 << 33)) != 0,
            up: (link & (1 << 34)) != 0,
        }
    }

    /// Start an Ethernet device.
    pub fn start(&self) -> Result<()> {
        rte_check!(unsafe { ffi::rte_eth_dev_start(self.0) })
    }

    /// Stop an Ethernet device.
    pub fn stop(&self) {
        unsafe { ffi::rte_eth_dev_stop(self.0) }
    }

    /// Close a stopped Ethernet device. The device cannot be restarted!
    pub fn close(&self) {
        unsafe { ffi::rte_eth_dev_close(self.0) }
    }
}

pub struct EthDeviceInfo(Box<ffi::Struct_rte_eth_dev_info>);

impl fmt::Debug for EthDeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "EthDeviceInfo {{ driver_name: \"{}\", if_index: {} }}",
               self.driver_name(),
               self.if_index())
    }
}

impl EthDeviceInfo {
    /// Device Driver name.
    pub fn driver_name(&self) -> &str {
        unsafe { CStr::from_ptr((*self.0).driver_name).to_str().unwrap() }
    }

    /// Index to bound host interface, or 0 if none. Use if_indextoname() to translate into an interface name.
    pub fn if_index(&self) -> u32 {
        (*self.0).if_index
    }

    pub fn pci_dev(&self) -> pci::RawDevicePtr {
        (*self.0).pci_dev
    }
}

/// A set of values to identify what method is to be used to route packets to multiple queues.
bitflags! {
    pub flags EthRxMultiQueueMode: u32 {
        const ETH_MQ_RX_RSS_FLAG    = 0x1,
        const ETH_MQ_RX_DCB_FLAG    = 0x2,
        const ETH_MQ_RX_VMDQ_FLAG   = 0x4,
    }
}

/// A structure used to configure the RX features of an Ethernet port.
pub struct EthRxMode {
    /// The multi-queue packet distribution mode to be used, e.g. RSS.
    pub mq_mode: EthRxMultiQueueMode,
    /// Header Split enable.
    pub split_hdr_size: u16,
    /// IP/UDP/TCP checksum offload enable.
    pub hw_ip_checksum: bool,
    /// VLAN filter enable.
    pub hw_vlan_filter: bool,
    /// VLAN strip enable.
    pub hw_vlan_strip: bool,
    /// Extended VLAN enable.
    pub hw_vlan_extend: bool,
    /// Jumbo Frame Receipt enable.
    pub max_rx_pkt_len: u32,
    /// Enable CRC stripping by hardware.
    pub hw_strip_crc: bool,
    /// Enable scatter packets rx handler
    pub enable_scatter: bool,
    /// Enable LRO
    pub enable_lro: bool,
}

impl Default for EthRxMode {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

/**
 * A set of values to identify what method is to be used to transmit
 * packets using multi-TCs.
 */
pub type EthTxMultiQueueMode = ffi::Enum_rte_eth_tx_mq_mode;

pub struct EthTxMode {
    /// TX multi-queues mode.
    pub mq_mode: EthTxMultiQueueMode,
    /// If set, reject sending out tagged pkts
    pub hw_vlan_reject_tagged: bool,
    /// If set, reject sending out untagged pkts
    pub hw_vlan_reject_untagged: bool,
    /// If set, enable port based VLAN insertion
    pub hw_vlan_insert_pvid: bool,
}

impl Default for EthTxMode {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

/// The RSS offload types are defined based on flow types which are defined
/// in rte_eth_ctrl.h. Different NIC hardwares may support different RSS offload
/// types. The supported flow types or RSS offload types can be queried by
/// rte_eth_dev_info_get().
bitflags! {
    pub flags RssHashFunc: u64 {
        const ETH_RSS_IPV4               = 1 << ::ffi::consts::RTE_ETH_FLOW_IPV4,
        const ETH_RSS_FRAG_IPV4          = 1 << ::ffi::consts::RTE_ETH_FLOW_FRAG_IPV4,
        const ETH_RSS_NONFRAG_IPV4_TCP   = 1 << ::ffi::consts::RTE_ETH_FLOW_NONFRAG_IPV4_TCP,
        const ETH_RSS_NONFRAG_IPV4_UDP   = 1 << ::ffi::consts::RTE_ETH_FLOW_NONFRAG_IPV4_UDP,
        const ETH_RSS_NONFRAG_IPV4_SCTP  = 1 << ::ffi::consts::RTE_ETH_FLOW_NONFRAG_IPV4_SCTP,
        const ETH_RSS_NONFRAG_IPV4_OTHER = 1 << ::ffi::consts::RTE_ETH_FLOW_NONFRAG_IPV4_OTHER,
        const ETH_RSS_IPV6               = 1 << ::ffi::consts::RTE_ETH_FLOW_IPV6,
        const ETH_RSS_FRAG_IPV6          = 1 << ::ffi::consts::RTE_ETH_FLOW_FRAG_IPV6,
        const ETH_RSS_NONFRAG_IPV6_TCP   = 1 << ::ffi::consts::RTE_ETH_FLOW_NONFRAG_IPV6_TCP,
        const ETH_RSS_NONFRAG_IPV6_UDP   = 1 << ::ffi::consts::RTE_ETH_FLOW_NONFRAG_IPV6_UDP,
        const ETH_RSS_NONFRAG_IPV6_SCTP  = 1 << ::ffi::consts::RTE_ETH_FLOW_NONFRAG_IPV6_SCTP,
        const ETH_RSS_NONFRAG_IPV6_OTHER = 1 << ::ffi::consts::RTE_ETH_FLOW_NONFRAG_IPV6_OTHER,
        const ETH_RSS_L2_PAYLOAD         = 1 << ::ffi::consts::RTE_ETH_FLOW_L2_PAYLOAD,
        const ETH_RSS_IPV6_EX            = 1 << ::ffi::consts::RTE_ETH_FLOW_IPV6_EX,
        const ETH_RSS_IPV6_TCP_EX        = 1 << ::ffi::consts::RTE_ETH_FLOW_IPV6_TCP_EX,
        const ETH_RSS_IPV6_UDP_EX        = 1 << ::ffi::consts::RTE_ETH_FLOW_IPV6_UDP_EX,
    }
}

pub struct EthRssConf {
    pub key: Option<[u8; 40]>,
    pub hash: RssHashFunc,
}

pub struct RxAdvConf {
    /// Port RSS configuration
    pub rss_conf: Option<EthRssConf>,
    pub vmdq_dcb_conf: Option<ffi::Struct_rte_eth_vmdq_dcb_conf>,
    pub dcb_rx_conf: Option<ffi::Struct_rte_eth_dcb_rx_conf>,
    pub vmdq_rx_conf: Option<ffi::Struct_rte_eth_vmdq_rx_conf>,
}

impl Default for RxAdvConf {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

pub enum TxAdvConf {

}

impl Default for TxAdvConf {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

/// Device supported speeds bitmap flags
bitflags! {
    pub flags LinkSpeed: u32 {
        /**< Autonegotiate (all speeds) */
        const ETH_LINK_SPEED_AUTONEG  = 0 <<  0,
        /**< Disable autoneg (fixed speed) */
        const ETH_LINK_SPEED_FIXED    = 1 <<  0,
        /**<  10 Mbps half-duplex */
        const ETH_LINK_SPEED_10M_HD   = 1 <<  1,
         /**<  10 Mbps full-duplex */
        const ETH_LINK_SPEED_10M      = 1 <<  2,
        /**< 100 Mbps half-duplex */
        const ETH_LINK_SPEED_100M_HD  = 1 <<  3,
        /**< 100 Mbps full-duplex */
        const ETH_LINK_SPEED_100M     = 1 <<  4,
        const ETH_LINK_SPEED_1G       = 1 <<  5,
        const ETH_LINK_SPEED_2_5G     = 1 <<  6,
        const ETH_LINK_SPEED_5G       = 1 <<  7,
        const ETH_LINK_SPEED_10G      = 1 <<  8,
        const ETH_LINK_SPEED_20G      = 1 <<  9,
        const ETH_LINK_SPEED_25G      = 1 << 10,
        const ETH_LINK_SPEED_40G      = 1 << 11,
        const ETH_LINK_SPEED_50G      = 1 << 12,
        const ETH_LINK_SPEED_56G      = 1 << 13,
        const ETH_LINK_SPEED_100G     = 1 << 14,
    }
}

pub struct EthConf {
    /// bitmap of ETH_LINK_SPEED_XXX of speeds to be used.
    ///
    /// ETH_LINK_SPEED_FIXED disables link autonegotiation, and a unique speed shall be set.
    /// Otherwise, the bitmap defines the set of speeds to be advertised.
    /// If the special value ETH_LINK_SPEED_AUTONEG (0) is used, all speeds supported are advertised.
    pub link_speeds: LinkSpeed,
    /// Port RX configuration.
    pub rxmode: Option<EthRxMode>,
    /// Port TX configuration.
    pub txmode: Option<EthTxMode>,
    /// Loopback operation mode.
    ///
    /// By default the value is 0, meaning the loopback mode is disabled.
    /// Read the datasheet of given ethernet controller for details.
    /// The possible values of this field are defined in implementation of each driver.
    pub lpbk_mode: u32,
    /// Port RX filtering configuration (union).
    pub rx_adv_conf: Option<RxAdvConf>,
    /// Port TX DCB configuration (union).
    pub tx_adv_conf: Option<TxAdvConf>,
    /// Currently,Priority Flow Control(PFC) are supported,
    /// if DCB with PFC is needed, and the variable must be set ETH_DCB_PFC_SUPPORT.
    pub dcb_capability_en: u32,
    pub fdir_conf: Option<ffi::Struct_rte_fdir_conf>,
    pub intr_conf: Option<ffi::Struct_rte_intr_conf>,
}

impl Default for EthConf {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

pub type RawEthConfPtr = *const ffi::Struct_rte_eth_conf;

pub struct RawEthConf(RawEthConfPtr);

impl RawEthConf {
    fn as_raw(&self) -> RawEthConfPtr {
        self.0
    }
}

impl Drop for RawEthConf {
    fn drop(&mut self) {
        unsafe { _rte_eth_conf_free(self.0) }
    }
}

impl<'a> From<&'a EthConf> for RawEthConf {
    fn from(c: &EthConf) -> Self {
        unsafe {
            let conf = _rte_eth_conf_new();

            if let Some(ref rxmode) = c.rxmode {
                _rte_eth_conf_set_rx_mode(conf,
                                          rxmode.mq_mode.bits,
                                          rxmode.split_hdr_size,
                                          rxmode.hw_ip_checksum as u8,
                                          rxmode.hw_vlan_filter as u8,
                                          rxmode.hw_vlan_strip as u8,
                                          rxmode.hw_vlan_extend as u8,
                                          rxmode.max_rx_pkt_len,
                                          rxmode.hw_strip_crc as u8,
                                          rxmode.enable_scatter as u8,
                                          rxmode.enable_lro as u8);
            }

            if let Some(ref txmode) = c.txmode {
                _rte_eth_conf_set_tx_mode(conf,
                                          txmode.mq_mode as u32,
                                          txmode.hw_vlan_reject_tagged as u8,
                                          txmode.hw_vlan_reject_untagged as u8,
                                          txmode.hw_vlan_insert_pvid as u8);
            }

            if let Some(ref adv_conf) = c.rx_adv_conf {
                if let Some(ref rss_conf) = adv_conf.rss_conf {
                    let (rss_key, rss_key_len) = rss_conf.key
                                                         .map_or_else(|| (ptr::null(), 0), |key| {
                                                             (key.as_ptr(), key.len() as u8)
                                                         });

                    _rte_eth_conf_set_rss_conf(conf, rss_key, rss_key_len, rss_conf.hash.bits);
                }
            }

            RawEthConf(conf)
        }
    }
}

pub type RawTxBufferPtr = *mut ffi::Struct_rte_eth_dev_tx_buffer;

///  Structure used to buffer packets for future TX
#[derive(Debug, PartialEq, Eq)]
pub struct TxBuffer(RawTxBufferPtr);

impl TxBuffer {
    pub fn as_raw(&self) -> RawTxBufferPtr {
        self.0
    }
}

impl Drop for TxBuffer {
    fn drop(&mut self) {
        malloc::free(self.0 as *mut c_void);

        self.0 = ptr::null_mut();
    }
}

pub type TxBufferErrorCallback<T> = fn(unsent: *mut *mut ffi::Struct_rte_mbuf,
                                       count: u16,
                                       userdata: &T);

impl From<RawTxBufferPtr> for TxBuffer {
    fn from(p: RawTxBufferPtr) -> Self {
        TxBuffer(p)
    }
}

impl TxBuffer {
    /// Initialize default values for buffered transmitting
    pub fn new(size: usize, socket_id: i32) -> Result<TxBuffer> {
        unsafe {
            let p = malloc::zmalloc_socket("tx_buffer",
                                           _rte_eth_tx_buffer_size(size),
                                           0,
                                           socket_id) as RawTxBufferPtr;

            if p.is_null() {
                Err(Error::OsError(libc::ENOMEM))
            } else {
                let ret = ffi::rte_eth_tx_buffer_init(p, size as u16);

                if ret != 0 {
                    Err(Error::OsError(ret))
                } else {
                    Ok(TxBuffer(p))
                }
            }
        }
    }

    /// Extract the raw pointer from an underlying object.
    pub fn as_raw(&self) -> RawTxBufferPtr {
        return self.0;
    }

    /// Consumes the TxBuffer, returning the wrapped raw pointer.
    pub fn into_raw(&self) -> RawTxBufferPtr {
        let p = self.0;

        mem::forget(self);

        return p;
    }

    /// Configure a callback for buffered packets which cannot be sent
    pub fn set_err_callback<T>(&self,
                               callback: Option<TxBufferErrorCallback<T>>,
                               userdata: Option<&T>)
                               -> Result<()> {
        rte_check!(unsafe {
            ffi::rte_eth_tx_buffer_set_err_callback(self.0,
                                                    mem::transmute(callback),
                                                    mem::transmute(userdata))
        })
    }

    /// Silently dropping unsent buffered packets.
    pub fn drop_err_packets(&self) -> Result<()> {
        rte_check!(unsafe {
            ffi::rte_eth_tx_buffer_set_err_callback(self.0,
                                                    Some(ffi::rte_eth_tx_buffer_drop_callback),
                                                    ptr::null_mut())
        })
    }

    /// Tracking unsent buffered packets.
    pub fn count_err_packets(&self) -> Result<()> {
        rte_check!(unsafe {
            ffi::rte_eth_tx_buffer_set_err_callback(self.0,
                                                    Some(ffi::rte_eth_tx_buffer_count_callback),
                                                    ptr::null_mut())
        })
    }
}

extern "C" {
    fn _rte_eth_conf_new() -> RawEthConfPtr;

    fn _rte_eth_conf_free(conf: RawEthConfPtr);

    fn _rte_eth_conf_set_rx_mode(conf: RawEthConfPtr,
                                 mq_mode: libc::uint32_t,
                                 split_hdr_size: libc::uint16_t,
                                 hw_ip_checksum: libc::uint8_t,
                                 hw_vlan_filter: libc::uint8_t,
                                 hw_vlan_strip: libc::uint8_t,
                                 hw_vlan_extend: libc::uint8_t,
                                 max_rx_pkt_len: libc::uint32_t,
                                 hw_strip_crc: libc::uint8_t,
                                 enable_scatter: libc::uint8_t,
                                 enable_lro: libc::uint8_t);

    fn _rte_eth_conf_set_tx_mode(conf: RawEthConfPtr,
                                 mq_mode: libc::uint32_t,
                                 hw_vlan_reject_tagged: libc::uint8_t,
                                 hw_vlan_reject_untagged: libc::uint8_t,
                                 hw_vlan_insert_pvid: libc::uint8_t);

    fn _rte_eth_conf_set_rss_conf(conf: RawEthConfPtr,
                                  rss_key: *const libc::uint8_t,
                                  rss_key_len: libc::uint8_t,
                                  rss_hf: libc::uint64_t);

    fn _rte_eth_tx_buffer_size(size: libc::size_t) -> libc::size_t;
}
