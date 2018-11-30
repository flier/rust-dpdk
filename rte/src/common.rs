#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
pub enum ProcType {
    Auto = -1,     // RTE_PROC_AUTO
    Primary = 0,   // RTE_PROC_PRIMARY
    Secondary = 1, // RTE_PROC_SECONDARY
    Invalid = 2,   // RTE_PROC_INVALID
}

pub mod log {
    use std::os::unix::io::AsRawFd;

    use cfile;

    use errors::{AsResult, Result};
    use ffi;

    /// SDK log type
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, PartialEq)]
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
    #[derive(Clone, Copy, Debug, PartialEq)]
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

}
