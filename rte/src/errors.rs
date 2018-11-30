use std::error;
use std::ffi;
use std::fmt;
use std::io;
use std::os::raw::c_int;
use std::result;

use errno::errno;

use ffi::rte_strerror;

pub trait AsResult {
    type Result;

    fn as_result(self) -> Result<Self::Result>;
}

impl AsResult for c_int {
    type Result = ();

    fn as_result(self) -> Result<Self::Result> {
        if self == 0 {
            Ok(())
        } else {
            Err(Error::RteError(self))
        }
    }
}

extern "C" {
    fn _rte_errno() -> i32;
}

macro_rules! rte_check {
    ( $ret:expr ) => {
        rte_check!($ret; ok => {()}; err => {$crate::errors::Error::RteError($ret)})
    };
    ( $ret:expr; ok => $ok:block) => {
        rte_check!($ret; ok => $ok; err => {$crate::errors::Error::RteError($ret)})
    };
    ( $ret:expr; err => $err:block) => {
        rte_check!($ret; ok => {()}; err => $err)
    };
    ( $ret:expr; ok => $ok:block; err => $err:block ) => {{
        if $ret >= 0 {
            Ok($ok)
        } else {
            Err($err)
        }
    }};

    ( $ret:expr, NonNull ) => {
        rte_check!($ret, NonNull; ok => {$ret}; err => {$crate::errors::Error::rte_error()})
    };
    ( $ret:expr, NonNull; ok => $ok:block) => {
        rte_check!($ret, NonNull; ok => $ok; err => {$crate::errors::Error::rte_error()})
    };
    ( $ret:expr, NonNull; err => $err:block) => {
        rte_check!($ret, NonNull; ok => {$ret}; err => $err)
    };
    ( $ret:expr, NonNull; ok => $ok:block; err => $err:block ) => {{
        if !$ret.is_null() {
            Ok($ok)
        } else {
            Err($err)
        }
    }};
}

#[derive(Debug)]
pub enum Error {
    RteError(i32),
    OsError(i32),
    IoError(io::Error),
    NulError(ffi::NulError),
}

impl Error {
    pub fn rte_error() -> Error {
        Error::RteError(unsafe { _rte_errno() })
    }

    pub fn os_error() -> Error {
        Error::OsError(errno().0 as i32)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::RteError(errno) => write!(f, "RTE error, {}", unsafe {
                ffi::CStr::from_ptr(rte_strerror(errno)).to_str().unwrap()
            }),
            &Error::OsError(ref errno) => write!(f, "OS error, {}", errno),
            &Error::IoError(ref err) => write!(f, "IO error, {}", err),
            _ => write!(f, "{}", error::Error::description(self)),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::RteError(_) => "RTE error",
            &Error::OsError(_) => "OS error",
            &Error::IoError(ref err) => error::Error::description(err),
            &Error::NulError(ref err) => error::Error::description(err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<ffi::NulError> for Error {
    fn from(err: ffi::NulError) -> Error {
        Error::NulError(err)
    }
}

pub type Result<T> = result::Result<T, Error>;
