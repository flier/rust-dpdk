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

/// Return the number of execution units (lcores) on the system.
pub fn count() -> usize {
    config::get_configuration().lcore_count()
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

/// Get a lcore's role.
pub fn role(lcore_id: u32) -> Role {
    unsafe { mem::transmute(ffi::rte_eal_lcore_role(lcore_id)) }
}

/// Test if an lcore is enabled.
pub fn enabled(lcore_id: u32) -> bool {
    role(lcore_id) == Role::Rte
}

/// Get the ID of the physical socket of the specified lcore
pub fn socket_id(lcore_id: u32) -> u32 {
    unsafe { ffi::lcore_config[lcore_id as usize].socket_id }
}

/// Get the id of the master lcore
pub fn master() -> u32 {
    config::get_configuration().master_lcore()
}
