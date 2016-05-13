use std::mem;

use ffi;

use errors::Result;

pub type LcoreFunc<T> = fn(&T) -> i32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LcoreState {
    Wait = 0,
    Running = 1,
    Finished = 2,
}

impl From<ffi::Enum_rte_lcore_state_t> for LcoreState {
    fn from(s: ffi::Enum_rte_lcore_state_t) -> Self {
        match s {
            ffi::Enum_rte_lcore_state_t::WAIT => LcoreState::Wait,
            ffi::Enum_rte_lcore_state_t::RUNNING => LcoreState::Running,
            ffi::Enum_rte_lcore_state_t::FINISHED => LcoreState::Finished,
        }
    }
}

/// Launch a function on another lcore.
pub fn remote_launch<T>(f: Option<LcoreFunc<T>>, arg: Option<&T>, slave_id: u32) -> Result<()> {
    rte_check!(unsafe {
        ffi::rte_eal_remote_launch(mem::transmute(f), mem::transmute(arg), slave_id)
    })
}

/// Launch a function on all lcores.
pub fn mp_remote_launch<T>(f: Option<LcoreFunc<T>>,
                           arg: Option<&T>,
                           skip_master: bool)
                           -> Result<()> {
    rte_check!(unsafe {
        ffi::rte_eal_mp_remote_launch(mem::transmute(f),
                                      mem::transmute(arg),
                                      if skip_master {
                                          ffi::Enum_rte_rmt_call_master_t::SKIP_MASTER
                                      } else {
                                          ffi::Enum_rte_rmt_call_master_t::CALL_MASTER
                                      })
    })
}

/// Get the state of the lcore identified by slave_id.
pub fn get_lcore_state(slave_id: u32) -> LcoreState {
    LcoreState::from(unsafe { ffi::rte_eal_get_lcore_state(slave_id) })
}

/// Wait until an lcore finishes its job.
///
/// To be executed on the MASTER lcore only.
///
/// If the slave lcore identified by the slave_id is in a FINISHED state,
/// switch to the WAIT state. If the lcore is in RUNNING state, wait until
/// the lcore finishes its job and moves to the FINISHED state.
///
pub fn wait_lcore(slave_id: u32) -> bool {
    unsafe { ffi::rte_eal_wait_lcore(slave_id) == 0 }
}

/// Wait until all lcores finish their jobs.
///
/// To be executed on the MASTER lcore only.
pub fn mp_wait_lcore() {
    unsafe { ffi::rte_eal_mp_wait_lcore() }
}
