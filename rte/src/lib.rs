#[macro_use]
extern crate log;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate cfile;
extern crate errno;
extern crate itertools;
extern crate libc;
extern crate rand;
extern crate time;
#[macro_use]
extern crate num_derive;
extern crate num_traits;

extern crate rte_sys as ffi;

#[macro_use]
pub mod errors;
#[macro_use]
pub mod macros;
#[macro_use]
mod common;

mod utils;

pub mod mempool;
#[macro_use]
pub mod mbuf;

pub mod bond;
pub mod ethdev;
pub mod kni;
pub mod pci;

pub mod arp;
pub mod ether;
pub mod ip;

#[macro_use]
pub mod cmdline;

pub use self::common::*;
pub use self::errors::{ErrorKind, Result, RteError};
pub use self::ethdev::PortId;
pub use self::ethdev::QueueId;
pub use self::memory::SocketId;
pub use ffi::*;

pub mod raw {
    pub use ffi::*;
}

#[cfg(test)]
mod tests;
