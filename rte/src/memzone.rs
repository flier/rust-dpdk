use ffi::raw::*;

pub struct RteMemoryZone(*const Struct_rte_memzone);

impl RteMemoryZone {
    pub fn from_ptr(zone: *const Struct_rte_memzone) -> RteMemoryZone {
        RteMemoryZone(zone)
    }
}
