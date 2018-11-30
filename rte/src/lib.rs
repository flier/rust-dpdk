#[macro_use]
extern crate log;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
extern crate cfile;
extern crate errno;
extern crate libc;
extern crate rand;

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
pub mod mempool;
pub mod memzone;
#[macro_use]
pub mod mbuf;
pub mod cycles;
pub mod eal;
pub mod launch;
pub mod lcore;
pub mod spinlock;

pub mod bond;
pub mod dev;
pub mod devargs;
pub mod ethdev;
pub mod kni;
pub mod pci;

pub mod arp;
pub mod ether;
pub mod ip;

#[macro_use]
pub mod cmdline;

pub use errors::{Error, Result};
pub use ethdev::PortId;
pub use ethdev::QueueId;
pub use ffi::*;
pub use lcore::LcoreId;
pub use memory::SocketId;

pub mod raw {
    pub use ffi::*;
}

#[cfg(test)]
mod tests;
