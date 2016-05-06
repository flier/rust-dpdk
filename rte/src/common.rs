use std::os::unix::io::AsRawFd;

use ffi::{size_t, FILE, rte_openlog_stream};

use errors::{Error, Result};
use cfile::{Stream, CFile};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
pub enum ProcType {
    Auto = -1, // RTE_PROC_AUTO
    Primary = 0, // RTE_PROC_PRIMARY
    Secondary = 1, // RTE_PROC_SECONDARY
    Invalid = 2, // RTE_PROC_INVALID
}

extern "C" {
    pub fn _rte_cache_line_size() -> size_t;
}

pub fn openlog_stream<S: AsRawFd>(s: &S) -> Result<CFile> {
    let f = try!(CFile::open_stream(s, "w"));

    if unsafe { rte_openlog_stream(f.stream() as *mut FILE) } != 0 {
        Err(Error::rte_error())
    } else {
        Ok(f)
    }
}
