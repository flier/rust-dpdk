use std::os::raw::{c_int, c_uchar, c_uint};

use libc::uint16_t;

pub use rte_sys::*;

/// Error number value, stored per-thread, which can be queried after
/// calls to certain functions to determine why those functions failed.
pub fn rte_errno() -> i32 {
    unsafe { rte_sys::_rte_errno() }
}
