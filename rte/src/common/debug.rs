use ffi;

/// Dump the stack of the calling core to the console.
pub fn dump_stack() {
    unsafe { ffi::rte_dump_stack() }
}

/// Dump the registers of the calling core to the console.
pub fn dump_registers() {
    unimplemented!("rte_dump_registers is unimplemented")
}

/// Provide notification of a critical non-recoverable error and stop.
#[macro_export]
macro_rules! rte_panic {
    ($fmt:expr, $($args:tt)*) => (
        unsafe { ffi::__rte_panic(concat!(file!(), ":", line!())), format_args!($fmt, $($args)*) }
    )
}
