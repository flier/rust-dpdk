use std::mem;
use std::ptr;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use ffi;

use errors::Result;

pub use common::*;
pub use config::*;
pub use cycles::*;
pub use launch::*;

extern "C" {
    fn _rte_version() -> *const c_char;

    fn devinitfn_pmd_af_packet_drv();
    fn devinitfn_bond_drv();
    fn devinitfn_rte_cxgbe_driver();
    fn devinitfn_pmd_igb_drv();
    fn devinitfn_pmd_igbvf_drv();
    fn devinitfn_em_pmd_drv();
    fn devinitfn_ena_pmd_drv();
    fn devinitfn_rte_enic_driver();
    fn devinitfn_rte_fm10k_driver();
    fn devinitfn_rte_i40e_driver();
    fn devinitfn_rte_i40evf_driver();
    fn devinitfn_rte_ixgbe_driver();
    fn devinitfn_rte_ixgbevf_driver();
    fn devinitfn_pmd_null_drv();
    fn devinitfn_cryptodev_null_pmd_drv();
    fn devinitfn_pmd_ring_drv();
    fn devinitfn_pmd_vhost_drv();
    fn devinitfn_rte_virtio_driver();
    fn devinitfn_rte_vmxnet3_driver();
}

unsafe fn init_pmd_drivers() {
    devinitfn_pmd_af_packet_drv();
    devinitfn_bond_drv();
    devinitfn_rte_cxgbe_driver();
    devinitfn_pmd_igb_drv();
    devinitfn_pmd_igbvf_drv();
    devinitfn_em_pmd_drv();
    devinitfn_ena_pmd_drv();
    devinitfn_rte_enic_driver();
    devinitfn_rte_fm10k_driver();
    devinitfn_rte_i40e_driver();
    devinitfn_rte_i40evf_driver();
    devinitfn_rte_ixgbe_driver();
    devinitfn_rte_ixgbevf_driver();
    devinitfn_pmd_null_drv();
    devinitfn_cryptodev_null_pmd_drv();
    devinitfn_pmd_ring_drv();
    devinitfn_pmd_vhost_drv();
    devinitfn_rte_virtio_driver();
    devinitfn_rte_vmxnet3_driver();
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
        ffi::rte_exit(code, CString::new(msg).unwrap().as_ptr());
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
