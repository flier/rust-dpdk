use std::os::raw::{c_int, c_void};

use ffi;

use errors::Result;
use lcore::LcoreId;

pub type LcoreFunc<T> = fn(Option<T>) -> i32;

struct LcoreContext<T> {
    callback: LcoreFunc<T>,
    arg: Option<T>,
}

unsafe extern "C" fn lcore_stub<T>(arg: *mut c_void) -> c_int {
    let ctxt = Box::from_raw(arg as *mut LcoreContext<T>);

    (ctxt.callback)(ctxt.arg)
}

/// Launch a function on another lcore.
pub fn remote_launch<T>(callback: LcoreFunc<T>, arg: Option<T>, slave_id: LcoreId) -> Result<()> {
    let ctxt = Box::into_raw(Box::new(LcoreContext::<T> { callback, arg })) as *mut c_void;

    rte_check!(unsafe { ffi::rte_eal_remote_launch(Some(lcore_stub::<T>), ctxt, slave_id) })
}

/// Launch a function on all lcores.
pub fn mp_remote_launch<T>(
    callback: LcoreFunc<T>,
    arg: Option<T>,
    skip_master: bool,
) -> Result<()> {
    let ctxt = Box::into_raw(Box::new(LcoreContext::<T> { callback, arg })) as *mut c_void;
    let call_master = if skip_master {
        ffi::rte_rmt_call_master_t::SKIP_MASTER
    } else {
        ffi::rte_rmt_call_master_t::CALL_MASTER
    };

    rte_check!(unsafe { ffi::rte_eal_mp_remote_launch(Some(lcore_stub::<T>), ctxt, call_master,) })
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
