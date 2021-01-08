use std::cmp;
use std::ffi::CStr;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;

use anyhow::{anyhow, Result};
use libc;

use ffi;

use errors::rte_error;
use ether;
use mbuf;
use mempool;
use pci;

/// Initialize and preallocate KNI subsystem
pub fn init(max_kni_ifaces: usize) -> Result<()> {
    if unsafe { ffi::rte_kni_init(max_kni_ifaces as u32) } == 0 {
        Ok(())
    } else {
        Err(anyhow!(rte_error()))
    }
}

/// Close KNI device.
pub fn close() {
    unsafe { ffi::rte_kni_close() }
}

/// Allocate KNI interface according to the port id, mbuf size, mbuf pool,
/// configurations and callbacks for kernel requests.
///
/// The KNI interface created in the kernel space is the net interface
/// the traditional Linux application talking to.
///
pub fn alloc(
    pktmbuf_pool: &mut mempool::RawMemoryPool,
    conf: &KniDeviceConf,
    opts: Option<&KniDeviceOps>,
) -> Result<KniDevice> {
    unsafe {
        let mut kni_conf = ffi::rte_kni_conf {
            name: mem::zeroed(),
            core_id: conf.core_id,
            group_id: conf.group_id,
            mbuf_size: conf.mbuf_size,
            addr: conf.pci_addr,
            id: conf.pci_id,
            _bitfield_1: ffi::rte_kni_conf::new_bitfield_1(conf.flags.bits),
            mac_addr: mem::transmute(conf.mac_addr.into_bytes()),
            mtu: conf.mtu,
            max_mtu: conf.mtu,
            min_mtu: conf.mtu,
        };

        ptr::copy(
            conf.name.as_ptr(),
            kni_conf.name.as_mut_ptr() as *mut u8,
            cmp::min(conf.name.len(), kni_conf.name.len() - 1),
        );

        let p = ffi::rte_kni_alloc(pktmbuf_pool, &kni_conf, mem::transmute(opts));

        rte_check!(p, NonNull; ok => { KniDevice(p)})
    }
}

bitflags! {
    pub struct KniFlag: u8 {
        const FORCE_BIND = 1;
    }
}

/// Structure for configuring KNI device.
pub struct KniDeviceConf<'a> {
    /// KNI name which will be used in relevant network device.
    /// Let the name as short as possible, as it will be part of memzone name.
    pub name: &'a str,
    /// Core ID to bind kernel thread on
    pub core_id: u32,
    /// Group ID
    pub group_id: u16,
    /// mbuf size
    pub mbuf_size: u32,

    pub pci_addr: pci::Addr,
    pub pci_id: pci::Id,

    /// Flag to bind kernel thread
    pub flags: KniFlag,

    pub mac_addr: ether::EtherAddr,
    pub mtu: u16,
}

impl<'a> Default for KniDeviceConf<'a> {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

/// Pointer to function of changing MTU
pub type ChangeMtuCallback = fn(port_id: u8, new_mut: libc::c_uint) -> libc::c_int;

/// Pointer to function of configuring network interface
pub type ConfigNetworkInterfaceCallback = fn(port_id: u8, if_up: u8) -> libc::c_int;

pub type KniDeviceOps = ffi::rte_kni_ops;

pub type RawKniDevice = ffi::rte_kni;
pub type RawKniDevicePtr = *mut ffi::rte_kni;

pub struct KniDevice(RawKniDevicePtr);

impl Drop for KniDevice {
    fn drop(&mut self) {
        self.release().expect("fail to release KNI device")
    }
}

impl Deref for KniDevice {
    type Target = RawKniDevice;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for KniDevice {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

impl KniDevice {
    pub fn from_raw(p: RawKniDevicePtr) -> Self {
        KniDevice(p)
    }

    /// Extract the raw pointer from an underlying object.
    pub fn as_raw(&self) -> RawKniDevicePtr {
        self.0
    }

    /// Consume the KniDevice, returning the raw pointer from an underlying object.
    pub fn into_raw(self) -> RawKniDevicePtr {
        self.0
    }

    pub fn release(&mut self) -> Result<()> {
        if self.0.is_null() {
            Ok(())
        } else {
            rte_check!(unsafe {
                ffi::rte_kni_release(self.0)
            }; ok => {
                self.0 = ptr::null_mut();
            })
        }
    }

    /// Get the KNI context of its name.
    pub fn get(name: &str) -> Result<KniDevice> {
        let p = unsafe { ffi::rte_kni_get(try!(to_cptr!(name))) };

        rte_check!(p, NonNull; ok => { KniDevice(p) })
    }

    /// Get the name given to a KNI device
    pub fn name(&self) -> &str {
        unsafe { CStr::from_ptr(ffi::rte_kni_get_name(self.0)).to_str().unwrap() }
    }

    /// It is used to handle the request mbufs sent from kernel space.
    ///
    /// Then analyzes it and calls the specific actions for the specific requests.
    /// Finally constructs the response mbuf and puts it back to the resp_q.
    ///
    pub fn handle_requests(&self) -> Result<&Self> {
        rte_check!(unsafe { ffi::rte_kni_handle_request(self.0) }; ok => { self })
    }

    /// Retrieve a burst of packets from a KNI interface.
    ///
    /// The retrieved packets are stored in rte_mbuf structures
    /// whose pointers are supplied in the array of mbufs,
    /// and the maximum number is indicated by num.
    /// It handles the freeing of the mbufs in the free queue of KNI interface.
    ///
    pub fn rx_burst(&self, mbufs: &mut [mbuf::RawMBufPtr]) -> usize {
        unsafe { ffi::rte_kni_rx_burst(self.0, mbufs.as_mut_ptr(), mbufs.len() as u32) as usize }
    }

    /// Send a burst of packets to a KNI interface.
    ///
    /// The packets to be sent out are stored in rte_mbuf structures
    /// whose pointers are supplied in the array of mbufs,
    /// and the maximum number is indicated by num.
    /// It handles allocating the mbufs for KNI interface alloc queue.
    ///
    pub fn tx_burst(&self, mbufs: &mut [mbuf::RawMBufPtr]) -> usize {
        unsafe { ffi::rte_kni_rx_burst(self.0, mbufs.as_mut_ptr(), mbufs.len() as u32) as usize }
    }

    /// Register KNI request handling for a specified port,
    /// and it can be called by master process or slave process.
    pub fn register_handlers(&self, opts: Option<&KniDeviceOps>) -> Result<&Self> {
        rte_check!(unsafe {
            ffi::rte_kni_register_handlers(self.0, mem::transmute(opts))
        }; ok => { self })
    }

    /// Unregister KNI request handling for a specified port.
    pub fn unregister_handlers(&self) -> Result<&Self> {
        rte_check!(unsafe { ffi::rte_kni_unregister_handlers(self.0) }; ok => { self })
    }
}
