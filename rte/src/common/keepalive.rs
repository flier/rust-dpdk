use std::mem;
use std::os::raw::{c_int, c_void};
use std::ptr::{self, NonNull};

use anyhow::Result;

use errors::AsResult;
use ffi::{
    self,
    rte_keepalive_state::{self, *},
};
use lcore;
use utils::AsRaw;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum State {
    Unused = RTE_KA_STATE_UNUSED,
    Alive = RTE_KA_STATE_ALIVE,
    Missing = RTE_KA_STATE_MISSING,
    Dead = RTE_KA_STATE_DEAD,
    Gone = RTE_KA_STATE_GONE,
    Dozing = RTE_KA_STATE_DOZING,
    Sleep = RTE_KA_STATE_SLEEP,
}

impl From<rte_keepalive_state::Type> for State {
    fn from(t: rte_keepalive_state::Type) -> Self {
        unsafe { mem::transmute(t) }
    }
}

/// Keepalive failure callback.
///
/// Receives a data pointer passed to rte_keepalive_create() and the id of the
/// failed core.
pub type FailureCallback<T> = fn(Option<T>, lcore::Id);

/// Keepalive relay callback.
///
///  Receives a data pointer passed to rte_keepalive_register_relay_callback(),
///  the id of the core for which state is to be forwarded, and details of the
///  current core state.
pub type RelayCallback<T> = fn(Option<T>, lcore::Id, State, u64);

pub type RawKeepalive = ffi::rte_keepalive;
pub type RawKeepalivePtr = *mut ffi::rte_keepalive;

#[repr(transparent)]
#[derive(Debug)]
pub struct Keepalive(NonNull<RawKeepalive>);

impl AsRaw for Keepalive {
    type Raw = RawKeepalive;

    fn as_raw(&self) -> *mut Self::Raw {
        self.0.as_ptr()
    }
}

pub fn create<T>(callback: FailureCallback<T>, arg: Option<T>) -> Result<Keepalive> {
    Keepalive::new(callback, arg)
}

impl Keepalive {
    pub fn new<T>(callback: FailureCallback<T>, arg: Option<T>) -> Result<Self> {
        let ctxt = Box::into_raw(Box::new(FailureContext { callback, arg }));

        unsafe { ffi::rte_keepalive_create(Some(failure_stub::<T>), ctxt as *mut _) }
            .as_result()
            .map(Keepalive)
    }

    /// Checks & handles keepalive state of monitored cores.
    pub fn dispatch_pings(&self) {
        unsafe { ffi::rte_keepalive_dispatch_pings(ptr::null_mut(), self.as_raw() as *mut _) }
    }

    /// Registers a core for keepalive checks.
    pub fn register_core(&self, core_id: lcore::Id) {
        unsafe { ffi::rte_keepalive_register_core(self.as_raw(), *core_id as i32) }
    }

    /// Per-core keepalive check.
    ///
    /// This function needs to be called from within the main process loop of the LCore to be checked.
    pub fn mark_alive(&self) {
        unsafe { ffi::rte_keepalive_mark_alive(self.as_raw()) }
    }

    /// Per-core sleep-time indication.
    ///
    /// If CPU idling is enabled, this function needs to be called from within
    /// the main process loop of the LCore going to sleep,
    /// in order to avoid the LCore being mis-detected as dead.
    pub fn mark_sleep(&self) {
        unsafe { ffi::rte_keepalive_mark_sleep(self.as_raw()) }
    }

    /// Registers a 'live core' callback.
    ///
    /// The complement of the 'dead core' callback. This is called when a
    /// core is known to be alive, and is intended for cases when an app
    /// needs to know 'liveness' beyond just knowing when a core has died.
    pub fn register_relay_callback<T>(&self, callback: RelayCallback<T>, arg: Option<T>) {
        let ctxt = Box::into_raw(Box::new(RelayContext { callback, arg }));

        unsafe { ffi::rte_keepalive_register_relay_callback(self.as_raw(), Some(relay_stub::<T>), ctxt as *mut _) }
    }
}

struct FailureContext<T> {
    callback: FailureCallback<T>,
    arg: Option<T>,
}

unsafe extern "C" fn failure_stub<T>(data: *mut c_void, id_core: c_int) {
    let ctxt = Box::from_raw(data as *mut FailureContext<T>);

    (ctxt.callback)(ctxt.arg, lcore::id(id_core as u32))
}

struct RelayContext<T> {
    callback: RelayCallback<T>,
    arg: Option<T>,
}

unsafe extern "C" fn relay_stub<T>(
    data: *mut c_void,
    id_core: c_int,
    core_state: rte_keepalive_state::Type,
    last_seen: u64,
) {
    let ctxt = Box::from_raw(data as *mut RelayContext<T>);

    (ctxt.callback)(ctxt.arg, lcore::id(id_core as u32), core_state.into(), last_seen)
}
