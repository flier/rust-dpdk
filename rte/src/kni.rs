use std::ptr;
use std::mem;
use std::cmp;
use std::ffi::{CStr, CString};

use libc;

use ffi;

use mempool;
use mbuf;
use pci;
use errors::Result;

/// Initialize and preallocate KNI subsystem
pub fn init(max_kni_ifaces: usize) {
    unsafe { ffi::rte_kni_init(max_kni_ifaces as u32) }
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
pub fn alloc(pktmbuf_pool: &mempool::RawMemoryPool,
             conf: &KniDeviceConf,
             opts: Option<&KniDeviceOps>)
             -> Result<KniDevice> {
    unsafe {
        let mut kni_conf = ffi::Struct_rte_kni_conf {
            name: mem::zeroed(),
            core_id: conf.core_id,
            group_id: conf.group_id,
            mbuf_size: conf.mbuf_size,
            addr: conf.pci_addr,
            id: conf.pci_id,
            _bindgen_bitfield_1_: conf.flags.bits,
        };

        ptr::copy(conf.name.as_ptr(),
                  kni_conf.name.as_mut_ptr() as *mut u8,
                  cmp::min(conf.name.len(), kni_conf.name.len() - 1));

        let p = ffi::rte_kni_alloc(pktmbuf_pool.as_raw(), &kni_conf, mem::transmute(opts));

        rte_check!(p, NonNull; ok => { KniDevice(p)})
    }
}

bitflags! {
    pub flags KniFlag: u8 {
        const FORCE_BIND = 1,
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

pub type KniDeviceOps = ffi::Struct_rte_kni_ops;

pub type RawDevicePtr = *mut ffi::Struct_rte_kni;

pub struct KniDevice(RawDevicePtr);

impl From<RawDevicePtr> for KniDevice {
    fn from(p: RawDevicePtr) -> Self {
        KniDevice(p)
    }
}

impl KniDevice {
    /// Extract the raw pointer from an underlying object.
    pub fn as_raw(&self) -> RawDevicePtr {
        return self.0;
    }

    pub fn release(&mut self) -> Result<()> {
        rte_check!(unsafe { ffi::rte_kni_release(self.0) })
    }

    /// Get the KNI context of its name.
    pub fn get(name: &str) -> Result<KniDevice> {
        let p = unsafe { ffi::rte_kni_get(try!(CString::new(name)).as_ptr()) };

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
    pub fn handle_requests(&self) -> Result<()> {
        rte_check!(unsafe { ffi::rte_kni_handle_request(self.0) })
    }

    /// Retrieve a burst of packets from a KNI interface.
    ///
    /// The retrieved packets are stored in rte_mbuf structures
    /// whose pointers are supplied in the array of mbufs,
    /// and the maximum number is indicated by num.
    /// It handles the freeing of the mbufs in the free queue of KNI interface.
    ///
    pub fn rx_burst(&self, mbufs: &mut [mbuf::RawMbufPtr]) -> usize {
        unsafe { ffi::rte_kni_rx_burst(self.0, mbufs.as_mut_ptr(), mbufs.len() as u32) as usize }
    }

    /// Send a burst of packets to a KNI interface.
    ///
    /// The packets to be sent out are stored in rte_mbuf structures
    /// whose pointers are supplied in the array of mbufs,
    /// and the maximum number is indicated by num.
    /// It handles allocating the mbufs for KNI interface alloc queue.
    ///
    pub fn tx_burst(&self, mbufs: &mut [mbuf::RawMbufPtr]) -> usize {
        unsafe { ffi::rte_kni_rx_burst(self.0, mbufs.as_mut_ptr(), mbufs.len() as u32) as usize }
    }

    /// Register KNI request handling for a specified port,
    /// and it can be called by master process or slave process.
    pub fn register_handlers(&self, opts: Option<&KniDeviceOps>) -> Result<()> {
        rte_check!(unsafe { ffi::rte_kni_register_handlers(self.0, mem::transmute(opts)) })
    }

    /// Unregister KNI request handling for a specified port.
    pub fn unregister_handlers(&self) -> Result<()> {
        rte_check!(unsafe { ffi::rte_kni_unregister_handlers(self.0) })
    }
}
