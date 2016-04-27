use std::mem::forget;
use std::ops::Drop;
use std::os::unix::io::AsRawFd;

use libc;

use ffi::raw::*;

use errors::{Error, Result};

pub struct CFile(*mut libc::FILE);

impl CFile {
    fn open_fd<S: AsRawFd>(s: S, mode: &str) -> Result<CFile> {
        let f = unsafe { libc::fdopen(s.as_raw_fd(), mode.as_ptr() as *const i8) };

        if f.is_null() {
            Err(Error::os_error())
        } else {
            Ok(CFile(f))
        }
    }
}

impl Drop for CFile {
    fn drop(&mut self) {
        unsafe {
            libc::fclose(self.0);
        }
    }
}

fn openlog_stream<S: AsRawFd>(s: S) -> Result<()> {
    let f = try!(CFile::open_fd(s, "w"));

    if unsafe { rte_openlog_stream(f.0 as *mut FILE) } != 0 {
        Err(Error::rte_error())
    } else {
        forget(f);

        Ok(())
    }
}
