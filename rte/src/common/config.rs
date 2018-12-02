use std::mem;
use std::ops::Deref;

use ffi;

use eal::ProcType;
use lcore;
use memzone;

pub type RawMemConfig = ffi::rte_mem_config;
pub type RawMemConfigPtr = *mut ffi::rte_mem_config;

/// the structure for the memory configuration for the RTE.
pub struct MemoryConfig(RawMemConfigPtr);

impl From<RawMemConfigPtr> for MemoryConfig {
    fn from(p: RawMemConfigPtr) -> Self {
        MemoryConfig(p)
    }
}

impl Deref for MemoryConfig {
    type Target = RawMemConfig;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl MemoryConfig {
    /// Number of channels (0 if unknown).
    pub fn nchannel(&self) -> u32 {
        self.nchannel
    }

    /// Number of ranks (0 if unknown).
    pub fn nrank(&self) -> u32 {
        self.nrank
    }

    /// Memzone descriptors.
    pub fn memzones(&self) -> Vec<memzone::MemoryZone> {
        (0..self.memzones.len)
            .map(|idx| unsafe { ffi::rte_fbarray_get(&self.memzones, idx) as *const _ })
            .map(|zone| memzone::from_raw(zone))
            .collect()
    }
}

pub type RawConfig = ffi::rte_config;
pub type RawConfigPtr = *mut ffi::rte_config;

/// The global RTE configuration structure.
pub struct Config(RawConfigPtr);

impl From<RawConfigPtr> for Config {
    fn from(p: RawConfigPtr) -> Self {
        Config(p)
    }
}

impl Deref for Config {
    type Target = RawConfig;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl Config {
    /// Id of the master lcore
    pub fn master_lcore(&self) -> lcore::Id {
        self.master_lcore.into()
    }

    /// Number of available logical cores.
    pub fn lcore_count(&self) -> usize {
        self.lcore_count as usize
    }

    /// Primary or secondary configuration
    pub fn process_type(&self) -> ProcType {
        unsafe { mem::transmute(self.process_type) }
    }

    /// State of cores.
    pub fn lcore_roles(&self) -> &'static [lcore::Role] {
        unsafe { mem::transmute(&self.lcore_role[..(*self.0).lcore_count as usize]) }
    }

    /// State of core.
    pub fn lcore_role(&self, lcore_id: lcore::Id) -> lcore::Role {
        self.lcore_role[usize::from(lcore_id)].into()
    }

    /// Memory configuration, which may be shared across multiple DPDK instances
    pub fn memory_config(&self) -> MemoryConfig {
        self.mem_config.into()
    }
}

/// Get the global configuration structure.
pub fn config() -> Config {
    unsafe { ffi::rte_eal_get_configuration().into() }
}
