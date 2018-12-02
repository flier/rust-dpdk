#[macro_use]
extern crate log;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate cfile;
extern crate time;
extern crate errno;
extern crate itertools;
extern crate libc;
extern crate rand;
#[macro_use]
extern crate num_derive;
extern crate num_traits;

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
pub mod keepalive;

#[macro_use]
pub mod malloc;
pub mod memory;
pub mod mempool;
pub mod memzone;
#[macro_use]
pub mod mbuf;
mod cycles;
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

pub use common::*;
pub use cycles::*;
pub use errors::{ErrorKind, Result, RteError};
pub use ethdev::PortId;
pub use ethdev::QueueId;
pub use ffi::*;
pub use memory::SocketId;

pub mod raw {
    pub use ffi::*;
}

#[cfg(test)]
mod tests;
