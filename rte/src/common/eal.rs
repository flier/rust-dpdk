use std::ffi::CStr;
use std::fmt;
use std::mem;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::ptr;

use anyhow::Result;

use ffi::{self, rte_proc_type_t::*};

use errors::{AsResult};
use utils::AsCString;

// pub use common::config;
pub use launch::{mp_remote_launch, mp_wait_lcore, remote_launch};

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive, ToPrimitive)]
pub enum ProcType {
    Auto = RTE_PROC_AUTO,
    Primary = RTE_PROC_PRIMARY,
    Secondary = RTE_PROC_SECONDARY,
    Invalid = RTE_PROC_INVALID,
}

extern "C" {
    // fn vdrvinitfn_pmd_af_packet_drv();
    // fn vdrvinitfn_bbdev_null_pmd_drv();
    // fn vdrvinitfn_pmd_bond_drv();
    // fn vdrvinitfn_cryptodev_caam_jr_drv();
    // fn vdrvinitfn_cryptodev_scheduler_pmd_drv();
    // fn vdrvinitfn_dpaa2_cmdif_drv();
    // fn vdrvinitfn_vdev_eventdev_dpaa2_pmd();
    // fn vdrvinitfn_vdev_eventdev_dpaa_pmd();
    // fn vdrvinitfn_evdev_dsw_pmd_drv();
    // fn vdrvinitfn_failsafe_drv();
    // fn vdrvinitfn_ifpga_cfg_driver();
    // fn vdrvinitfn_eth_kni_drv();
    // fn vdrvinitfn_pmd_null_drv();
    // fn vdrvinitfn_cryptodev_null_pmd_drv();
    // fn vdrvinitfn_octeontx_pmd_drv();
    // fn vdrvinitfn_vdev_ssovf_pmd();
    // fn vdrvinitfn_evdev_opdl_pmd_drv();
    // fn vdrvinitfn_pmd_ring_drv();
    // fn vdrvinitfn_vdev_eventdev_skeleton_pmd();
    // fn vdrvinitfn_skeleton_pmd_drv();
    // fn vdrvinitfn_pmd_softnic_drv();
    // fn vdrvinitfn_evdev_sw_pmd_drv();
    // fn vdrvinitfn_pmd_tap_drv();
    // fn vdrvinitfn_pmd_tun_drv();
    // fn vdrvinitfn_vdev_netvsc_vdev();
    // fn vdrvinitfn_pmd_vhost_drv();
    // fn vdrvinitfn_virtio_user_driver();
}

unsafe fn init_pmd_drivers() {
    // vdrvinitfn_pmd_af_packet_drv();
    // vdrvinitfn_bbdev_null_pmd_drv();
    // vdrvinitfn_pmd_bond_drv();
    // vdrvinitfn_cryptodev_caam_jr_drv();
    // vdrvinitfn_cryptodev_scheduler_pmd_drv();
    // vdrvinitfn_dpaa2_cmdif_drv();
    // vdrvinitfn_vdev_eventdev_dpaa2_pmd();
    // vdrvinitfn_vdev_eventdev_dpaa_pmd();
    // vdrvinitfn_evdev_dsw_pmd_drv();
    // vdrvinitfn_failsafe_drv();
    // vdrvinitfn_ifpga_cfg_driver();
    // vdrvinitfn_eth_kni_drv();
    // vdrvinitfn_pmd_null_drv();
    // vdrvinitfn_cryptodev_null_pmd_drv();
    // vdrvinitfn_octeontx_pmd_drv();
    // vdrvinitfn_vdev_ssovf_pmd();
    // vdrvinitfn_evdev_opdl_pmd_drv();
    // vdrvinitfn_pmd_ring_drv();
    // vdrvinitfn_vdev_eventdev_skeleton_pmd();
    // vdrvinitfn_skeleton_pmd_drv();
    // vdrvinitfn_pmd_softnic_drv();
    // vdrvinitfn_evdev_sw_pmd_drv();
    // vdrvinitfn_pmd_tap_drv();
    // vdrvinitfn_pmd_tun_drv();
    // vdrvinitfn_vdev_netvsc_vdev();
    // vdrvinitfn_pmd_vhost_drv();
    // vdrvinitfn_virtio_user_driver();
}

/// Request iopl privilege for all RPL.
pub fn iopl_init() -> Result<()> {
    unsafe { ffi::rte_eal_iopl_init() }.as_result().map(|_| ())
}

/// Initialize the Environment Abstraction Layer (EAL).
///
/// This function is to be executed on the MASTER lcore only,
/// as soon as possible in the application's main() function.
///
/// The function finishes the initialization process before main() is called.
/// It puts the SLAVE lcores in the WAIT state.
///
pub fn init<S: fmt::Debug + AsRef<str>>(args: &[S]) -> Result<i32> {
    debug!("initial EAL with {} args: {:?}", args.len(), args);

    // rust doesn't support __attribute__((constructor)), we need to invoke those static initializer
    unsafe {
        init_pmd_drivers();
    }

    let parsed = if args.is_empty() {
        unsafe { ffi::rte_eal_init(0, ptr::null_mut()) }
    } else {
        let args: Vec<_> = args.iter().map(|s| s.as_cstring()).collect();
        let mut cptrs: Vec<_> = args.iter().map(|s| s.as_ptr() as *mut c_char).collect();

        unsafe { ffi::rte_eal_init(cptrs.len() as i32, cptrs.as_mut_ptr()) }
    };

    debug!("EAL parsed {} arguments", parsed);

    parsed.as_result().map(|_| parsed)
}

/// Clean up the Environment Abstraction Layer (EAL)
pub fn cleanup() -> Result<()> {
    unsafe { ffi::rte_eal_cleanup() }.as_result().map(|_| ())
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

/// Whether EAL is using PCI bus.
pub fn has_pci() -> bool {
    unsafe { ffi::rte_eal_has_pci() != 0 }
}

/// Whether the EAL was asked to create UIO device.
pub fn create_uio_dev() -> bool {
    unsafe { ffi::rte_eal_create_uio_dev() != 0 }
}

/// Get the runtime directory of DPDK
pub fn runtime_dir() -> PathBuf {
    PathBuf::from(unsafe {
        CStr::from_ptr(ffi::rte_eal_get_runtime_dir())
            .to_string_lossy()
            .into_owned()
    })
}
