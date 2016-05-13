use std::time::Duration;

use ffi;

/// Get the measured frequency of the RDTSC counter
pub fn get_tsc_hz() -> u64 {
    unsafe { ffi::rte_get_tsc_hz() }
}

/// Wait at least us microseconds.
pub fn delay_us(us: u32) {
    unsafe { ffi::rte_delay_us(us) }
}

/// Wait at least ms milliseconds.
pub fn delay_ms(ms: u32) {
    delay_us(ms * 1000)
}

pub fn delay(d: Duration) {
    delay_us(d.as_secs() as u32 * 1000_000 + d.subsec_nanos() / 1000)
}
