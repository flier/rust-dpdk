#[macro_use]
extern crate log;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate errno;
extern crate cfile;

extern crate rte_sys as ffi;

#[macro_use]
pub mod errors;
pub mod common;
pub mod config;
#[macro_use]
pub mod malloc;
pub mod memzone;
pub mod mempool;
pub mod mbuf;
pub mod net;
pub mod lcore;
pub mod cycles;
pub mod launch;
pub mod eal;
pub mod ethdev;
pub mod pci;
pub mod kni;

pub use errors::{Error, Result};
pub use ffi::consts::*;

pub mod raw {
    pub use ffi::*;
}

#[cfg(test)]
mod tests;
