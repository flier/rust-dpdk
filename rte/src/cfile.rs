use std::io;
use std::ops::Drop;
use std::ops::Deref;
use std::os::unix::io::{AsRawFd, RawFd};

use libc;

pub struct CFile(*mut libc::FILE);

impl CFile {
    pub fn from_raw(f: *mut libc::FILE) -> io::Result<CFile> {
        if f.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(CFile(f))
        }
    }

    pub fn open_stream<S: AsRawFd>(s: &S, mode: &str) -> io::Result<CFile> {
        Self::open_fd(s.as_raw_fd(), mode)
    }

    pub fn open_fd(fd: RawFd, mode: &str) -> io::Result<CFile> {
        Self::from_raw(unsafe { libc::fdopen(fd, mode.as_ptr() as *const i8) })
    }

    pub fn open_stdin() -> io::Result<CFile> {
        Self::open_fd(libc::STDIN_FILENO, "r")
    }

    pub fn open_stdout() -> io::Result<CFile> {
        Self::open_fd(libc::STDOUT_FILENO, "w")
    }

    pub fn open_stderr() -> io::Result<CFile> {
        Self::open_fd(libc::STDERR_FILENO, "w")
    }

    pub fn open_tmpfile() -> io::Result<CFile> {
        Self::from_raw(unsafe { libc::tmpfile() })
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

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::{Read, Write, Seek, SeekFrom};
    use std::os::unix::io::AsRawFd;

    use super::*;

    #[test]
    fn test_cfile() {
        let mut f = CFile::open_tmpfile().unwrap();

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
