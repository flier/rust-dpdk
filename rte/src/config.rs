use std::mem;

use ffi;

use common::ProcType;
use lcore;
use memzone;

pub type RawMemoryConfig = ffi::rte_mem_config;
pub type RawMemoryConfigPtr = *mut RawMemoryConfig;

/// the structure for the memory configuration for the RTE.
pub struct MemoryConfig(RawMemoryConfigPtr);

impl MemoryConfig {
    fn from_ptr(cfg: RawMemoryConfigPtr) -> MemoryConfig {
        MemoryConfig(cfg)
    }

    /// Number of channels (0 if unknown).
    pub fn nchannel(&self) -> u32 {
        unsafe { (*self.0).nchannel }
    }

    /// Number of ranks (0 if unknown).
    pub fn nrank(&self) -> u32 {
        unsafe { (*self.0).nrank }
    }

    /// Memzone descriptors.
    pub fn memzones(&self) -> Vec<memzone::MemoryZone> {
        unsafe {
            Vec::from(&(*self.0).memzone[..(*self.0).memzone_cnt as usize])
                .iter()
                .map(|zone| memzone::from_raw(zone))
                .collect()
        }
    }
}

/// The global RTE configuration structure.
pub struct RteConfig(*mut ffi::rte_config);

impl RteConfig {
    fn from_ptr(cfg: *mut ffi::rte_config) -> RteConfig {
        RteConfig(cfg)
    }

    /// Id of the master lcore
    pub fn master_lcore(&self) -> u32 {
        unsafe { (*self.0).master_lcore }
    }

    /// Number of available logical cores.
    pub fn lcore_count(&self) -> usize {
        unsafe { (*self.0).lcore_count as usize }
    }

    /// Primary or secondary configuration
    pub fn process_type(&self) -> ProcType {
        unsafe { mem::transmute((*self.0).process_type) }
    }

    /// State of cores.
    pub fn lcore_roles(&self) -> &'static [lcore::Role] {
        unsafe { mem::transmute(&(*self.0).lcore_role[..(*self.0).lcore_count as usize]) }
    }

    /// Memory configuration, which may be shared across multiple DPDK instances
    pub fn memory_config(&self) -> MemoryConfig {
        MemoryConfig::from_ptr(unsafe { (*self.0).mem_config })
    }
}

/// Get the global configuration structure.
pub fn get_configuration() -> RteConfig {
    unsafe { RteConfig::from_ptr(ffi::rte_eal_get_configuration()) }
}
