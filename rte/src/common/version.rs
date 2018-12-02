use std::ffi::CStr;

use ffi;

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
