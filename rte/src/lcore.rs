//! API for lcore and socket manipulation
//!
use std::mem;
use std::ops::Deref;

use ffi;

use config::config;
use errors::{rte_error, Result};
use memory::SocketId;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Id(u32);

impl<T: Into<u32>> From<T> for Id {
    fn from(id: T) -> Self {
        Id(id.into())
    }
}

impl Deref for Id {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Id> for usize {
    fn from(id: Id) -> Self {
        id.0 as usize
    }
}

impl Id {
    /// Any lcore.
    pub fn any() -> Id {
        Id(ffi::LCORE_ID_ANY)
    }

    pub fn max() -> Id {
        Id(ffi::RTE_MAX_LCORE)
    }

    /// Get the ID of the physical socket of the specified lcore
    pub fn socket_id(self) -> SocketId {
        unsafe { ffi::lcore_config[self.0 as usize].socket_id as SocketId }
    }

    /// Test if an lcore is enabled.
    pub fn is_enabled(self) -> bool {
        config().lcore_role(self) == Role::Rte
    }

    /// All the enabled lcores.
    pub fn enabled() -> Vec<Id> {
        foreach_lcores(false).collect()
    }

    /// Get the id of the master lcore
    pub fn master() -> Id {
        config().master_lcore()
    }

    pub fn is_master(self) -> bool {
        self.0 == config().master_lcore().0
    }

    /// Return the index of the lcore starting from zero.
    pub fn index(self) -> usize {
        unsafe { ffi::lcore_config[self.0 as usize].core_index as usize }
    }

    /// Test if the core supplied has a specific role
    pub fn has_role(self, role: Role) -> bool {
        unsafe { ffi::rte_lcore_has_role(self.0, role as u32) == 0 }
    }

    /// Get a lcore's role.
    pub fn role(self) -> Role {
        config().lcore_role(self)
    }

    /// Get the state of the lcore identified by lcore_id.
    pub fn state(self) -> State {
        unsafe { ffi::rte_eal_get_lcore_state(self.0) }.into()
    }

    /// Get the next enabled lcore ID.
    pub fn next(self, skip_master: bool, wrap: bool) -> Option<Id> {
        let mut next_id = self.0;

        loop {
            next_id += 1;

            if wrap {
                next_id %= ffi::RTE_MAX_LCORE;
            } else if next_id >= ffi::RTE_MAX_LCORE || next_id == self.0 {
                return None;
            }

            if !Id(next_id).is_enabled() {
                continue;
            }

            if skip_master && Id(next_id).is_master() {
                continue;
            }

            break;
        }

        Some(Id(next_id))
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive, ToPrimitive)]
pub enum Role {
    Rte = ffi::rte_lcore_role_t::ROLE_RTE,
    Off = ffi::rte_lcore_role_t::ROLE_OFF,
    Service = ffi::rte_lcore_role_t::ROLE_SERVICE,
}

impl From<u32> for Role {
    fn from(role: u32) -> Self {
        unsafe { mem::transmute(role) }
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum State {
    Wait = ffi::rte_lcore_state_t::WAIT,
    Running = ffi::rte_lcore_state_t::RUNNING,
    Finished = ffi::rte_lcore_state_t::FINISHED,
}

impl From<ffi::rte_lcore_state_t::Type> for State {
    fn from(s: ffi::rte_lcore_state_t::Type) -> Self {
        unsafe { mem::transmute(s) }
    }
}

extern "C" {
    pub fn _rte_lcore_id() -> u32;
}

/// Return the ID of the execution unit we are running on.
pub fn id() -> Option<Id> {
    match unsafe { _rte_lcore_id() } {
        ffi::LCORE_ID_ANY => None,
        id @ _ => Some(id.into()),
    }
}

/// Return the number of execution units (lcores) on the system.
pub fn count() -> usize {
    config().lcore_count()
}

/// Return the ID of the physical socket of the logical core we are running on.
pub fn socket_id() -> u32 {
    unsafe { ffi::rte_socket_id() }
}

/// Return number of physical sockets detected on the system.
///
/// Note that number of nodes may not be correspondent to their physical id's:
/// for example, a system may report two socket id's, but the actual socket id's
/// may be 0 and 8.
pub fn socket_count() -> u32 {
    unsafe { ffi::rte_socket_count() }
}

/// Return socket id with a particular index.
///
/// This will return socket id at a particular position in list of all detected
/// physical socket id's. For example, on a machine with sockets [0, 8], passing
/// 1 as a parameter will return 8.
pub fn socket_id_by_idx(idx: u32) -> Result<SocketId> {
    let id = unsafe { ffi::rte_socket_id_by_idx(idx) };

    if id < 0 {
        Err(rte_error())
    } else {
        Ok(id)
    }
}

/// Browse all running lcores.
pub fn foreach<T, F: FnMut(Id)>(f: F) {
    foreach_lcores(false).for_each(f)
}

/// Browse all running lcores except the master lcore.
pub fn foreach_slave<T, F: FnMut(Id)>(f: F) {
    foreach_lcores(true).for_each(f)
}

fn foreach_lcores(skip_master: bool) -> impl Iterator<Item = Id> {
    (0..ffi::RTE_MAX_LCORE)
        .map(Id)
        .filter(|lcore_id| lcore_id.is_enabled())
        .filter(|lcore_id| !skip_master || !lcore_id.is_master())
}
