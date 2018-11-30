use std::os::unix::io::AsRawFd;

use ffi::{rte_openlog_stream, FILE};

use cfile;
use errors::Result;
use libc;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
pub enum ProcType {
    Auto = -1,     // RTE_PROC_AUTO
    Primary = 0,   // RTE_PROC_PRIMARY
    Secondary = 1, // RTE_PROC_SECONDARY
    Invalid = 2,   // RTE_PROC_INVALID
}

extern "C" {
    pub fn _rte_cache_line_size() -> libc::size_t;
}

pub fn openlog_stream<S: AsRawFd>(s: &S) -> Result<cfile::CFile> {
    let f = try!(cfile::open_stream(s, "w"));

    rte_check!(unsafe { rte_openlog_stream(f.stream() as *mut FILE) }; ok => {f})
}
