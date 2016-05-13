use std::time::Duration;

use ffi;

/// Get the measured frequency of the RDTSC counter
#[inline]
pub fn get_tsc_hz() -> u64 {
    unsafe { ffi::rte_get_tsc_hz() }
}

/// Wait at least us microseconds.
#[inline]
pub fn delay_us(us: u32) {
    unsafe { ffi::rte_delay_us(us) }
}

/// Wait at least ms milliseconds.
#[inline]
pub fn delay_ms(ms: u32) {
    delay_us(ms * 1000)
}

#[inline]
pub fn delay(d: Duration) {
    delay_us(d.as_secs() as u32 * 1000_000 + d.subsec_nanos() / 1000)
}

extern "C" {
    fn _rte_rdtsc() -> u64;

    fn _rte_rdtsc_precise() -> u64;
}

#[inline]
pub fn rdtsc() -> u64 {
    unsafe { _rte_rdtsc() }
}

#[inline]
pub fn rdtsc_precise() -> u64 {
    unsafe { _rte_rdtsc_precise() }
}
