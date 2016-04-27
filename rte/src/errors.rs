use std::fmt;
use std::error;
use std::result;
use std::ffi;

use errno::{Errno, errno};

use ffi::raw::rte_strerror;

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum RteErrno {
    MinErrno = 1000, // RTE_MIN_ERRNO
    Secondary = 1001, // E_RTE_SECONDARY
    NoConfig = 1002, // E_RTE_NO_CONFIG
    MaxErrno = 1003, // RTE_MAX_ERRNO
}

extern "C" {
    fn _rte_errno() -> i32;
}

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    RteError(i32),
    OsError(Errno),
    NulError(ffi::NulError),
}

impl Error {
    pub fn rte_error() -> Error {
        Error::RteError(unsafe { _rte_errno() })
    }

    pub fn os_error() -> Error {
        Error::OsError(errno())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::RteError(errno) => {
                write!(f,
                       "RTE error, {}",
                       unsafe { ffi::CStr::from_ptr(rte_strerror(errno)).to_str().unwrap() })
            }
            &Error::OsError(err) => write!(f, "OS error, {}", err),
            _ => write!(f, "{}", error::Error::description(self).to_string()),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::RteError(_) => "RTE error.",
            &Error::OsError(ref err) => "OS error.",
            &Error::NulError(ref err) => error::Error::description(err),
        }
    }
}

impl From<ffi::NulError> for Error {
    fn from(err: ffi::NulError) -> Error {
        Error::NulError(err)
    }
}

pub type Result<T> = result::Result<T, Error>;
