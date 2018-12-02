//!  Pseudo-random Generators in RTE

use ffi;

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
