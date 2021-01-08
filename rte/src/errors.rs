use std::ffi::CStr;
use std::fmt;
use std::os::raw::c_int;
use std::ptr::NonNull;
use std::result;

use anyhow::{anyhow, Result};
use errno::errno;

use ffi;

pub trait AsResult {
    type Result;

    fn as_result(self) -> Result<Self::Result>;

    fn ok_or<E: std::error::Error>(self, err: E) -> Result<Self::Result>;

    fn ok_or_else<E: std::error::Error, F: FnOnce() -> E>(self, err: F) -> Result<Self::Result>;
}

impl<T> AsResult for *mut T {
    type Result = NonNull<T>;

    fn as_result(self) -> Result<Self::Result> {
        NonNull::new(self).ok_or_else(|| anyhow!(rte_error()))
    }

    fn ok_or<E: std::error::Error>(self, err: E) -> Result<Self::Result> {
        NonNull::new(self).ok_or_else(|| anyhow!("{}", err))
    }

    fn ok_or_else<E: std::error::Error, F: FnOnce() -> E>(self, err: F) -> Result<Self::Result> {
        NonNull::new(self).ok_or_else(|| anyhow!("{}", err()))
    }
}

impl AsResult for c_int {
    type Result = c_int;

    fn as_result(self) -> Result<Self::Result> {
        if self == -1 {
            Err(anyhow!(RteError(self)))
        } else {
            Ok(self)
        }
    }

    fn ok_or<E: std::error::Error>(self, err: E) -> Result<Self::Result> {
        if self == -1 {
            Err(anyhow!("{}", err))
        } else {
            Ok(self)
        }
    }

    fn ok_or_else<E: std::error::Error, F: FnOnce() -> E>(self, err: F) -> Result<Self::Result> {
        if self == -1 {
            Err(anyhow!("{}", err()))
        } else {
            Ok(self)
        }
    }
}

macro_rules! rte_check {
    ( $ret:expr ) => {
        rte_check!($ret; ok => {()}; err => {anyhow::anyhow!($crate::errors::RteError($ret))})
    };
    ( $ret:expr; ok => $ok:block) => {
        rte_check!($ret; ok => $ok; err => {anyhow::anyhow!($crate::errors::RteError($ret))})
    };
    ( $ret:expr; err => $err:block) => {
        rte_check!($ret; ok => {()}; err => $err)
    };
    ( $ret:expr; ok => $ok:block; err => $err:block ) => {{
        if $ret == 0 {
            Ok($ok)
        } else {
            Err($err)
        }
    }};

    ( $ret:expr, NonNull ) => {
        rte_check!($ret, NonNull; ok => {$ret}; err => {anyhow::anyhow!($crate::errors::rte_error())})
    };
    ( $ret:expr, NonNull; ok => $ok:block) => {
        rte_check!($ret, NonNull; ok => $ok; err => {anyhow::anyhow!($crate::errors::rte_error())})
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
pub struct RteError(pub i32);

impl fmt::Display for RteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "RTE error, {} ({})",
            unsafe { CStr::from_ptr(ffi::rte_strerror(self.0)).to_string_lossy() },
            self.0
        )
    }
}

impl std::error::Error for RteError {}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("invalid log type, {0}")]
    InvalidLogType(u32),
    #[error("invalid log level, {0}")]
    InvalidLogLevel(u32),
    #[error("cmdline parse error, {0}")]
    CmdLineParseError(i32),
    #[error("{0}")]
    OsError(i32),
}

pub fn rte_error() -> RteError {
    RteError(unsafe { ffi::rte_errno() })
}

pub fn os_error() -> ErrorKind {
    ErrorKind::OsError(errno().0 as i32)
}
