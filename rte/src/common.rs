use std::ffi::{CStr, CString};

use errors::{AsResult, Result};
use ffi;

#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive, ToPrimitive)]
#[repr(i32)]
pub enum ProcType {
    Auto = -1,     // RTE_PROC_AUTO
    Primary = 0,   // RTE_PROC_PRIMARY
    Secondary = 1, // RTE_PROC_SECONDARY
    Invalid = 2,   // RTE_PROC_INVALID
}

pub trait AsCString {
    fn as_cstring(&self) -> CString;
}

impl<T> AsCString for T
where
    T: AsRef<str>,
{
    fn as_cstring(&self) -> CString {
        let mut v = self.as_ref().as_bytes().to_owned();
        v.push(0);
        unsafe { CString::from_vec_unchecked(v) }
    }
}

/// Patch level number i.e. the z in yy.mm.z
pub use ffi::RTE_VER_MINOR;
/// Minor version/month number i.e. the mm in yy.mm.z
pub use ffi::RTE_VER_MONTH;
/// Patch release number
pub use ffi::RTE_VER_RELEASE;
/// Major version/year number i.e. the yy in yy.mm.z
pub use ffi::RTE_VER_YEAR;

/// Macro to compute a version number usable for comparisons
macro_rules! RTE_VERSION_NUM {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        (($a) << 24 | ($b) << 16 | ($c) << 8 | ($d))
    };
}

lazy_static! {
    /// String that appears before the version number
    pub static ref RTE_VER_PREFIX: &'static str = unsafe { CStr::from_bytes_with_nul_unchecked(ffi::RTE_VER_PREFIX).to_str().unwrap() };
    /// Extra string to be appended to version number
    pub static ref RTE_VER_SUFFIX: &'static str = unsafe { CStr::from_bytes_with_nul_unchecked(ffi::RTE_VER_SUFFIX).to_str().unwrap() };

    /// All version numbers in one to compare with RTE_VERSION_NUM()
    pub static ref RTE_VERSION: u32 =
        RTE_VERSION_NUM!(RTE_VER_YEAR, RTE_VER_MONTH, RTE_VER_MINOR, RTE_VER_RELEASE);

    pub static ref RTE_VERSION_STR: String = version();
}

/// Function returning version string
pub fn version() -> String {
    if ffi::RTE_VER_SUFFIX.is_empty() {
        format!(
            "{} {}.{:02}.{}",
            *RTE_VER_PREFIX, RTE_VER_YEAR, RTE_VER_MONTH, RTE_VER_MINOR
        )
    } else {
        format!(
            "{} {}.{:02}.{}{}{}",
            *RTE_VER_PREFIX,
            RTE_VER_YEAR,
            RTE_VER_MONTH,
            RTE_VER_MINOR,
            *RTE_VER_SUFFIX,
            if RTE_VER_RELEASE < 16 {
                RTE_VER_RELEASE
            } else {
                RTE_VER_RELEASE - 16
            }
        )
    }
}

/// RTE Logs API
pub mod log {
    use std::mem;
    use std::os::unix::io::AsRawFd;

    use cfile;

    use common::AsCString;
    use errors::{AsResult, ErrorKind::*, Result};
    use ffi;

    /// SDK log type
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, PartialEq, FromPrimitive, ToPrimitive)]
    pub enum Type {
        /// Log related to eal.
        Eal = ffi::RTE_LOGTYPE_EAL,
        /// Log related to malloc.
        Malloc = ffi::RTE_LOGTYPE_MALLOC,
        /// Log related to ring.
        Ring = ffi::RTE_LOGTYPE_RING,
        /// Log related to mempool.
        MemPool = ffi::RTE_LOGTYPE_MEMPOOL,
        /// Log related to timers.
        Timer = ffi::RTE_LOGTYPE_TIMER,
        /// Log related to poll mode driver.
        PMD = ffi::RTE_LOGTYPE_PMD,
        /// Log related to hash table.
        Hash = ffi::RTE_LOGTYPE_HASH,
        /// Log related to LPM.
        LPM = ffi::RTE_LOGTYPE_LPM,
        /// Log related to KNI.
        KNI = ffi::RTE_LOGTYPE_KNI,
        /// Log related to ACL.
        ACL = ffi::RTE_LOGTYPE_ACL,
        /// Log related to power.
        Power = ffi::RTE_LOGTYPE_POWER,
        /// Log related to QoS meter.
        Meter = ffi::RTE_LOGTYPE_METER,
        /// Log related to QoS port scheduler.
        PortScheduler = ffi::RTE_LOGTYPE_SCHED,
        /// Log related to port.
        Port = ffi::RTE_LOGTYPE_PORT,
        /// Log related to table.
        Table = ffi::RTE_LOGTYPE_TABLE,
        /// Log related to pipeline.
        Pipeline = ffi::RTE_LOGTYPE_PIPELINE,
        /// Log related to mbuf.
        MBuf = ffi::RTE_LOGTYPE_MBUF,
        /// Log related to cryptodev.
        CryptoDev = ffi::RTE_LOGTYPE_CRYPTODEV,
        /// Log related to EFD.
        EFD = ffi::RTE_LOGTYPE_EFD,
        /// Log related to eventdev.
        EventDev = ffi::RTE_LOGTYPE_EVENTDEV,
        /// Log related to GSO.
        GSO = ffi::RTE_LOGTYPE_GSO,
        /// User-defined log type 1.
        User1 = ffi::RTE_LOGTYPE_USER1,
        /// User-defined log type 2.
        User2 = ffi::RTE_LOGTYPE_USER2,
        /// User-defined log type 3.
        User3 = ffi::RTE_LOGTYPE_USER3,
        /// User-defined log type 4.
        User4 = ffi::RTE_LOGTYPE_USER4,
        /// User-defined log type 5.
        User5 = ffi::RTE_LOGTYPE_USER5,
        /// User-defined log type 6.
        User6 = ffi::RTE_LOGTYPE_USER6,
        /// User-defined log type 7.
        User7 = ffi::RTE_LOGTYPE_USER7,
        /// User-defined log type 8.
        User8 = ffi::RTE_LOGTYPE_USER8,

        /// First identifier for extended logs
        FirstExt = ffi::RTE_LOGTYPE_FIRST_EXT_ID,
    }

    #[repr(u32)]
    #[derive(Clone, Copy, Debug, PartialEq, FromPrimitive, ToPrimitive)]
    pub enum Level {
        /// System is unusable.
        Emerge = ffi::RTE_LOG_EMERG,
        /// Action must be taken immediately.
        Alert = ffi::RTE_LOG_ALERT,
        /// Critical conditions.
        Critical = ffi::RTE_LOG_CRIT,
        /// Error conditions.
        Error = ffi::RTE_LOG_ERR,
        /// Warning conditions.
        Warn = ffi::RTE_LOG_WARNING,
        /// Normal but significant condition.
        Notice = ffi::RTE_LOG_NOTICE,
        /// Informational.
        Info = ffi::RTE_LOG_INFO,
        /// Debug-level messages.
        Debug = ffi::RTE_LOG_DEBUG,
    }

    /// Change the stream that will be used by the logging system.
    ///
    /// This can be done at any time. The f argument represents the stream
    /// to be used to send the logs. If f is NULL, the default output is
    /// used (stderr).
    pub fn openlog_stream<S: AsRawFd>(s: &S) -> Result<cfile::CFile> {
        let f = cfile::open_stream(s, "w")?;

        unsafe { ffi::rte_openlog_stream(f.stream() as *mut ffi::FILE) }
            .as_result()
            .map(|_| f)
    }

    /// Set the global log level.
    ///
    /// After this call, logs with a level lower or equal than the level
    /// passed as argument will be displayed.
    pub fn set_global_level(level: Level) {
        unsafe { ffi::rte_log_set_global_level(level as u32) }
    }

    /// Get the global log level.
    pub fn get_global_level() -> Level {
        unsafe { mem::transmute(ffi::rte_log_get_global_level()) }
    }

    /// Get the log level for a given type.
    pub fn get_level(ty: Type) -> Result<Level> {
        unsafe { ffi::rte_log_get_level(ty as u32) }
            .ok_or(InvalidLogType(ty as u32))
            .map(|level| unsafe { mem::transmute(level) })
    }

    /// Set the log level for a given type.
    pub fn set_level(ty: Type, level: Level) -> Result<()> {
        unsafe { ffi::rte_log_set_level(ty as u32, level as u32) }
            .ok_or(InvalidLogLevel(level as u32))
            .map(|_| ())
    }

    /// Get the current loglevel for the message being processed.
    ///
    /// Before calling the user-defined stream for logging, the log
    /// subsystem sets a per-lcore variable containing the loglevel and the
    /// logtype of the message being processed. This information can be
    /// accessed by the user-defined log output function through this function.
    pub fn cur_msg_loglevel() -> Level {
        unsafe { mem::transmute(ffi::rte_log_cur_msg_loglevel()) }
    }

    /// Get the current logtype for the message being processed.
    ///
    /// Before calling the user-defined stream for logging, the log
    /// subsystem sets a per-lcore variable containing the loglevel and the
    /// logtype of the message being processed. This information can be
    /// accessed by the user-defined log output function through this function.
    pub fn cur_msg_logtype() -> Type {
        unsafe { mem::transmute(ffi::rte_log_cur_msg_logtype()) }
    }

    /// Register a dynamic log type
    ///
    /// If a log is already registered with the same type, the returned value
    /// is the same than the previous one.
    pub fn register<S: AsRef<str>>(name: S) -> Result<()> {
        let name = name.as_cstring();

        unsafe { ffi::rte_log_register(name.as_ptr()) }
            .as_result()
            .map(|_| ())
    }

    /// Dump log information.
    ///
    /// Dump the global level and the registered log types.
    pub fn dump<S: AsRawFd>(s: &S) -> Result<()> {
        let f = cfile::open_stream(s, "w")?;

        unsafe { ffi::rte_log_dump(f.stream() as *mut ffi::FILE) };

        Ok(())
    }
}

/// Generates a log message.
///
/// The message will be sent in the stream defined by the previous call
/// to rte_openlog_stream().
///
/// The level argument determines if the log should be displayed or
/// not, depending on the global rte_logs variable.
///
/// The preferred alternative is the RTE_LOG() because it adds the
/// level and type in the logged string.
pub fn log(level: log::Level, ty: log::Type, msg: &str) -> Result<()> {
    let msg = msg.as_cstring();

    unsafe { ffi::rte_log(level as u32, ty as u32, msg.as_ptr()) }
        .as_result()
        .map(|_| ())
}

//  Pseudo-random Generators in RTE

/// Seed the pseudo-random generator.
///
/// The generator is automatically seeded by the EAL init with a timer
/// value. It may need to be re-seeded by the user with a real random value.
pub fn srand(seed: i64) {
    unsafe { ffi::srand48(seed) }
}

/// Get a pseudo-random value.
///
/// This function generates pseudo-random numbers using the linear
/// congruential algorithm and 48-bit integer arithmetic, called twice
/// to generate a 64-bit value.
pub fn rand() -> u64 {
    unsafe {
        let mut v = ffi::lrand48() as u64;
        v <<= 32;
        v += ffi::lrand48() as u64;
        v
    }
}
