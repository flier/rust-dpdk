use std::io;
use std::mem::forget;
use std::ops::Drop;
use std::ops::Deref;
use std::os::unix::io::{AsRawFd, RawFd};

use libc;

use ffi::{uint32_t, size_t, FILE, rte_openlog_stream};

use errors::{Error, Result};

extern "C" {
    pub fn _rte_lcore_id() -> uint32_t;

    pub fn _rte_cache_line_size() -> size_t;
}

pub struct CFile(*mut libc::FILE);

impl CFile {
    pub fn from_raw(f: *mut libc::FILE) -> Result<CFile> {
        if f.is_null() {
            Err(Error::os_error())
        } else {
            Ok(CFile(f))
        }
    }

    pub fn open_fd<S: AsRawFd>(s: &S, mode: &str) -> Result<CFile> {
        CFile::from_raw(unsafe { libc::fdopen(s.as_raw_fd(), mode.as_ptr() as *const i8) })
    }

    pub fn new_tmpfile() -> Result<CFile> {
        CFile::from_raw(unsafe { libc::tmpfile() })
    }
}

impl Drop for CFile {
    fn drop(&mut self) {
        unsafe {
            libc::fclose(self.0);
        }
    }
}

impl Deref for CFile {
    type Target = *mut libc::FILE;

    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

impl AsRawFd for CFile {
    fn as_raw_fd(&self) -> RawFd {
        unsafe { libc::fileno(self.0) }
    }
}

impl io::Read for CFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        unsafe {
            let read = libc::fread(buf.as_ptr() as *mut libc::c_void, 1, buf.len(), self.0);

            if read <= 0 {
                if libc::feof(self.0) != 0 {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read to EOF"));
                }

                let errno = libc::ferror(self.0);

                if errno != 0 {
                    return Err(io::Error::from_raw_os_error(errno));
                }
            }

            Ok(read)
        }
    }
}

impl io::Write for CFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unsafe {
            let wrote = libc::fwrite(buf.as_ptr() as *const libc::c_void, 1, buf.len(), self.0);

            if wrote <= 0 {
                let errno = libc::ferror(self.0);

                if errno != 0 {
                    return Err(io::Error::from_raw_os_error(errno));
                }
            }

            Ok(wrote)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        unsafe {
            if libc::fflush(self.0) == 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}

impl io::Seek for CFile {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        unsafe {
            let ret = match pos {
                io::SeekFrom::Start(off) => libc::fseek(self.0, off as i64, libc::SEEK_SET),
                io::SeekFrom::End(off) => libc::fseek(self.0, off, libc::SEEK_END),
                io::SeekFrom::Current(off) => libc::fseek(self.0, off, libc::SEEK_CUR),
            };

            if ret == 0 {
                let off = libc::ftell(self.0);

                if off >= 0 {
                    return Ok(off as u64);
                }
            }

            Err(io::Error::last_os_error())
        }
    }
}

pub fn openlog_stream<S: AsRawFd>(s: &S) -> Result<()> {
    let f = try!(CFile::open_fd(s, "w"));

    if unsafe { rte_openlog_stream(f.0 as *mut FILE) } != 0 {
        Err(Error::rte_error())
    } else {
        forget(f);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::{Read, Write, Seek, SeekFrom};
    use std::os::unix::io::AsRawFd;

    use super::*;

    #[test]
    fn test_cfile() {
        let mut f = CFile::new_tmpfile().unwrap();

        assert!(!(*f).is_null());
        assert!(f.as_raw_fd() > 2);

        assert_eq!(f.write(b"test").unwrap(), 4);
        assert_eq!(f.seek(SeekFrom::Current(0)).unwrap(), 4);

        let mut buf: [u8; 4] = [0; 4];

        assert_eq!(f.read(&mut buf[..]).err().unwrap().kind(),
                   io::ErrorKind::UnexpectedEof);

        assert_eq!(f.seek(SeekFrom::Start(0)).unwrap(), 0);

        f.flush().unwrap();

        assert_eq!(f.read(&mut buf[..]).unwrap(), 4);
        assert_eq!(buf, &b"test"[..]);
    }
}
