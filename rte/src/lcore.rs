use std::mem;

use ffi;

use config;
use memory::SocketId;

pub type LcoreId = u32;

pub const LCORE_ID_ANY: LcoreId = !0 as u32;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum Role {
    Rte = 0, // ROLE_RTE
    Off = 1, // ROLE_OFF
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Wait = 0,
    Running = 1,
    Finished = 2,
}

impl From<ffi::Enum_rte_lcore_state_t> for State {
    fn from(s: ffi::Enum_rte_lcore_state_t) -> Self {
        match s {
            ffi::Enum_rte_lcore_state_t::WAIT => State::Wait,
            ffi::Enum_rte_lcore_state_t::RUNNING => State::Running,
            ffi::Enum_rte_lcore_state_t::FINISHED => State::Finished,
        }
    }
}

extern "C" {
    pub fn _rte_lcore_id() -> ffi::uint32_t;
}

/// Return the ID of the execution unit we are running on.
pub fn id() -> Option<LcoreId> {
    match unsafe { _rte_lcore_id() } {
        LCORE_ID_ANY => None,
        id @ _ => Some(id),
    }
}

/// Get the id of the master lcore
pub fn master() -> LcoreId {
    config::get_configuration().master_lcore()
}

/// Return the number of execution units (lcores) on the system.
pub fn count() -> usize {
    config::get_configuration().lcore_count()
}

/// Return the index of the lcore starting from zero.
pub fn index(lcore_id: LcoreId) -> Option<u32> {
    match lcore_id {
        LCORE_ID_ANY => id(),
        0...ffi::RTE_MAX_LCORE => {
            Some(unsafe { ffi::lcore_config[lcore_id as usize].core_index as u32 })
        }
        _ => None,
    }
}

/// Get the next enabled lcore ID.
pub fn next(lcore_id: LcoreId, skip_master: bool) -> LcoreId {
    let mut i = (lcore_id + 1) % ffi::RTE_MAX_LCORE;

    while i < ffi::RTE_MAX_LCORE {
        if !is_enabled(i) || (skip_master && i == master()) {
            i = (i + 1) % ffi::RTE_MAX_LCORE;

            continue;
        }

        break;
    }

    i
}

/// Get a lcore's role.
pub fn role(lcore_id: LcoreId) -> Role {
    unsafe { mem::transmute(ffi::rte_eal_lcore_role(lcore_id)) }
}

/// Get the state of the lcore identified by lcore_id.
pub fn state(lcore_id: LcoreId) -> State {
    State::from(unsafe { ffi::rte_eal_get_lcore_state(lcore_id) })
}

/// Get the ID of the physical socket of the specified lcore
pub fn socket_id(lcore_id: LcoreId) -> SocketId {
    unsafe { ffi::lcore_config[lcore_id as usize].socket_id as SocketId }
}

/// Test if an lcore is enabled.
pub fn is_enabled(lcore_id: LcoreId) -> bool {
    role(lcore_id) == Role::Rte
}

#[inline]
pub fn foreach<T, F: Fn(LcoreId) -> T>(f: F) -> Vec<T> {
    foreach_lcores(f, false)
}

#[inline]
pub fn foreach_slave<T, F: Fn(LcoreId) -> T>(f: F) -> Vec<T> {
    foreach_lcores(f, true)
}

pub fn foreach_lcores<T, F: Fn(LcoreId) -> T>(f: F, skip_master: bool) -> Vec<T> {
    let master_lcore = config::get_configuration().master_lcore();

    (0..ffi::RTE_MAX_LCORE)
        .filter(|lcore_id| is_enabled(*lcore_id))
        .filter(|lcore_id| !skip_master || (*lcore_id != master_lcore))
        .map(|lcore_id| f(lcore_id))
        .collect()
}

#[inline]
pub fn enabled_lcores() -> Vec<LcoreId> {
    foreach(|lcore_id| lcore_id)
}
