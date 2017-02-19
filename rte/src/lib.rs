#[macro_use]
extern crate log;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate rand;
extern crate errno;
extern crate cfile;

extern crate rte_sys as ffi;

#[macro_use]
pub mod errors;
#[macro_use]
pub mod macros;
#[macro_use]
pub mod byteorder;
pub mod common;
#[macro_use]
pub mod debug;
pub mod config;

#[macro_use]
pub mod malloc;
pub mod memory;
pub mod memzone;
pub mod mempool;
#[macro_use]
pub mod mbuf;
pub mod lcore;
pub mod cycles;
pub mod spinlock;
#[macro_use]
pub mod launch;
pub mod eal;

pub mod devargs;
pub mod ethdev;
pub mod pci;
pub mod kni;
pub mod bond;

pub mod ether;
pub mod arp;
pub mod ip;

#[macro_use]
pub mod cmdline;

pub use errors::{Error, Result};
pub use memory::SocketId;
pub use lcore::LcoreId;
pub use ethdev::PortId;
pub use ethdev::QueueId;

pub mod raw {
    pub use ffi::*;
}

pub use ffi::{RTE_MAX_LCORE, RTE_MAX_ETHPORTS};

#[cfg(test)]
mod tests;
