use std::mem::forget;
use std::os::unix::io::AsRawFd;

use ffi::{uint32_t, size_t, FILE, rte_openlog_stream};

use errors::{Error, Result};
use cfile::CFile;

extern "C" {
    pub fn _rte_lcore_id() -> uint32_t;

    pub fn _rte_cache_line_size() -> size_t;
}

pub fn openlog_stream<S: AsRawFd>(s: &S) -> Result<()> {
    let f = try!(CFile::open_fd(s, "w"));

    if unsafe { rte_openlog_stream(*f as *mut FILE) } != 0 {
        Err(Error::rte_error())
    } else {
        forget(f);

        Ok(())
    }
}
