use std::fmt;
use std::error;
use std::result;
use std::ffi;

use errno::{Errno, errno};

use ffi::rte_strerror;

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
            &Error::OsError(ref errno) => write!(f, "OS error, {}", errno),
            _ => write!(f, "{}", error::Error::description(self)),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::RteError(_) => "RTE error",
            &Error::OsError(_) => "OS error",
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
