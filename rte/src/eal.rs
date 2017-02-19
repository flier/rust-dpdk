use std::mem;
use std::ptr;
use std::ffi::CStr;
use std::os::raw::c_char;

use ffi;

use mempool;
use errors::Result;

pub use common::*;
pub use config::*;
pub use cycles::*;
pub use launch::*;

extern "C" {
    fn _rte_version() -> *const c_char;
}

unsafe fn init_pmd_drivers() {
    // ffi::vdrvinitfn_pmd_af_packet_drv();
    // ffi::vdrvinitfn_bond_drv();
    // ffi::vdrvinitfn_pmd_null_drv();
    // ffi::vdrvinitfn_cryptodev_null_pmd_drv();
    // ffi::vdrvinitfn_pmd_ring_drv();
    // ffi::vdrvinitfn_pmd_vhost_drv();
    // ffi::vdrvinitfn_virtio_user_driver();
    // ffi::pciinitfn_net_cxgbe();
    // ffi::pciinitfn_net_e1000_igb();
    // ffi::pciinitfn_net_e1000_igb_vf();
    // ffi::pciinitfn_net_e1000_em();
    // ffi::pciinitfn_net_ena();
    // ffi::pciinitfn_net_enic();
    // ffi::pciinitfn_net_fm10k();
    // ffi::pciinitfn_net_i40e();
    // ffi::pciinitfn_net_i40e_vf();
    // ffi::pciinitfn_net_ixgbe();
    // ffi::pciinitfn_net_ixgbe_vf();
    // ffi::pciinitfn_net_qede();
    // ffi::pciinitfn_net_qede_vf();
    // ffi::pciinitfn_net_vmxnet3();
}

pub fn version<'a>() -> &'a str {
    unsafe { CStr::from_ptr(_rte_version()).to_str().unwrap() }
}

/// Initialize the Environment Abstraction Layer (EAL).
///
/// This function is to be executed on the MASTER lcore only,
/// as soon as possible in the application's main() function.
///
/// The function finishes the initialization process before main() is called.
/// It puts the SLAVE lcores in the WAIT state.
///
pub fn init(args: &Vec<String>) -> Result<i32> {
    debug!("initial EAL with {} args: {:?}",
           args.len(),
           args.as_slice());

    // rust doesn't support __attribute__((constructor)), we need to invoke those static initializer
    unsafe {
        init_pmd_drivers();
        mempool::init();
    }

    let parsed = if args.is_empty() {
        unsafe { ffi::rte_eal_init(0, ptr::null_mut()) }
    } else {
        let cargs: Vec<Vec<u8>> = args.iter()
            .map(|s| {
                let mut v: Vec<u8> = Vec::from(s.as_bytes());
                v.push(0);
                v
            })
            .collect();

        let mut cptrs: Vec<*mut c_char> = cargs.iter()
            .map(|s| s.as_ptr() as *mut c_char)
            .collect();

        unsafe { ffi::rte_eal_init(cptrs.len() as i32, cptrs.as_mut_ptr()) }
    };

    debug!("EAL parsed {} arguments", parsed);

    rte_check!(parsed; ok => { parsed })
}

/// Function to terminate the application immediately,
/// printing an error message and returning the exit_code back to the shell.
pub fn exit(code: i32, msg: &str) {
    unsafe {
        ffi::rte_exit(code, to_cptr!(msg).unwrap());
    }
}

/// Get the process type in a multi-process setup
pub fn process_type() -> ProcType {
    unsafe { mem::transmute(ffi::rte_eal_process_type()) }
}

/// Check if a primary process is currently alive
pub fn primary_proc_alive() -> bool {
    unsafe { ffi::rte_eal_primary_proc_alive(ptr::null()) != 0 }
}

/// Whether EAL is using huge pages (disabled by --no-huge option).
pub fn has_hugepages() -> bool {
    unsafe { ffi::rte_eal_has_hugepages() != 0 }
}

/// Return the ID of the physical socket of the logical core we are running on.
pub fn socket_id() -> i32 {
    unsafe { ffi::rte_socket_id() as i32 }
}
