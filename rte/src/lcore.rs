use std::mem;

use ffi;

use config;

const LCORE_ID_ANY: u32 = 0xffffffff;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum Role {
    Rte = 0, // ROLE_RTE
    Off = 1, // ROLE_OFF
}

extern "C" {
    pub fn _rte_lcore_id() -> ffi::uint32_t;
}

/// Return the ID of the execution unit we are running on.
pub fn id() -> Option<u32> {
    let id = unsafe { _rte_lcore_id() };

    if id == LCORE_ID_ANY {
        None
    } else {
        Some(id)
    }
}

/// Get the id of the master lcore
pub fn master() -> u32 {
    config::get_configuration().master_lcore()
}

/// Return the number of execution units (lcores) on the system.
pub fn count() -> usize {
    config::get_configuration().lcore_count()
}

/// Return the index of the lcore starting from zero.
pub fn index(lcore_id: i32) -> Option<u32> {
    if lcore_id >= ffi::RTE_MAX_LCORE as i32 {
        None
    } else if lcore_id < 0 {
        id()
    } else {
        Some(unsafe { ffi::lcore_config[lcore_id as usize].core_index as u32 })
    }
}

/// Get a lcore's role.
pub fn role(lcore_id: u32) -> Role {
    unsafe { mem::transmute(ffi::rte_eal_lcore_role(lcore_id)) }
}

/// Get the ID of the physical socket of the specified lcore
pub fn socket_id(lcore_id: u32) -> u32 {
    unsafe { ffi::lcore_config[lcore_id as usize].socket_id }
}

/// Test if an lcore is enabled.
pub fn is_enabled(lcore_id: u32) -> bool {
    role(lcore_id) == Role::Rte
}

#[inline]
pub fn foreach<T, F: Fn(u32) -> T>(f: F) -> Vec<T> {
    foreach_lcores(f, false)
}

#[inline]
pub fn foreach_slave<T, F: Fn(u32) -> T>(f: F) -> Vec<T> {
    foreach_lcores(f, true)
}

pub fn foreach_lcores<T, F: Fn(u32) -> T>(f: F, skip_master: bool) -> Vec<T> {
    let master_lcore = config::get_configuration().master_lcore();

    (0..ffi::RTE_MAX_LCORE)
        .filter(|lcore_id| is_enabled(*lcore_id))
        .filter(|lcore_id| !skip_master || (*lcore_id != master_lcore))
        .map(|lcore_id| f(lcore_id))
        .collect()
}

#[inline]
pub fn enabled_lcores() -> Vec<u32> {
    foreach(|lcore_id| lcore_id)
}
