use std::mem;

use ffi;

use errors::Result;
use lcore::LcoreId;

pub type LcoreFunc<T> = extern "C" fn(Option<&T>) -> i32;

#[macro_export]
macro_rules! lcore_func {
    ($func:ident) => (unsafe { mem::transmute($func as extern "C" fn(_) -> i32) })
}

/// Launch a function on another lcore.
pub fn remote_launch<T>(f: LcoreFunc<T>, arg: Option<&T>, slave_id: LcoreId) -> Result<()> {
    let ret =
        unsafe { ffi::rte_eal_remote_launch(mem::transmute(f), mem::transmute(arg), slave_id) };

    rte_check!(ret)
}

/// Launch a function on all lcores.
pub fn mp_remote_launch<T>(f: LcoreFunc<T>, arg: Option<&T>, skip_master: bool) -> Result<()> {
    let ret = unsafe {
        ffi::rte_eal_mp_remote_launch(mem::transmute(f),
                                      mem::transmute(arg),
                                      if skip_master {
                                          ffi::rte_rmt_call_master_t::SKIP_MASTER
                                      } else {
                                          ffi::rte_rmt_call_master_t::CALL_MASTER
                                      })
    };

    rte_check!(ret)
}

/// Wait until an lcore finishes its job.
///
/// To be executed on the MASTER lcore only.
///
/// If the slave lcore identified by the slave_id is in a FINISHED state,
/// switch to the WAIT state. If the lcore is in RUNNING state, wait until
/// the lcore finishes its job and moves to the FINISHED state.
///
pub fn wait_lcore(lcore_id: LcoreId) -> bool {
    unsafe { ffi::rte_eal_wait_lcore(lcore_id) == 0 }
}

/// Wait until all lcores finish their jobs.
///
/// To be executed on the MASTER lcore only.
pub fn mp_wait_lcore() {
    unsafe { ffi::rte_eal_mp_wait_lcore() }
}
