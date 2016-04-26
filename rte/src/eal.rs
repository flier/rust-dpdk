use std::mem;
use std::ffi::CString;
use std::os::raw::c_char;

use ffi::raw::*;

use errors::*;
use memzone::RteMemoryZone;

/// the structure for the memory configuration for the RTE.
pub struct RteMemoryConfig(*mut Struct_rte_mem_config);

impl RteMemoryConfig {
    pub fn from_ptr(cfg: *mut Struct_rte_mem_config) -> RteMemoryConfig {
        RteMemoryConfig(cfg)
    }

    /// Number of channels (0 if unknown).
    fn nchannel(&self) -> u32 {
        unsafe { (*self.0).nchannel }
    }

    /// Number of ranks (0 if unknown).
    fn nrank(&self) -> u32 {
        unsafe { (*self.0).nrank }
    }

    /// Memzone descriptors.
    fn memzones(&self) -> Vec<RteMemoryZone> {
        unsafe {
            Vec::from(&(*self.0).memzone[..(*self.0).memzone_cnt as usize])
                .iter()
                .map(|zone| RteMemoryZone::from_ptr(zone))
                .collect()
        }
    }
}

pub type RteProcType = Enum_rte_proc_type_t;

pub type RteLcoreRole = Enum_rte_lcore_role_t;

/// The global RTE configuration structure.
pub struct RteConfig(*mut Struct_rte_config);

impl RteConfig {
    pub fn from_ptr(cfg: *mut Struct_rte_config) -> RteConfig {
        RteConfig(cfg)
    }

    /// Id of the master lcore
    fn master_lcore(&self) -> u32 {
        unsafe { (*self.0).master_lcore }
    }

    /// Number of available logical cores.
    fn lcore_count(&self) -> usize {
        unsafe { (*self.0).lcore_count as usize }
    }

    /// Primary or secondary configuration
    fn process_type(&self) -> RteProcType {
        unsafe { (*self.0).process_type as RteProcType }
    }

    /// State of cores.
    fn lcore_roles(&self) -> &'static [RteLcoreRole] {
        unsafe { &(*self.0).lcore_role[..(*self.0).lcore_count as usize] }
    }

    /// Memory configuration, which may be shared across multiple DPDK instances
    fn memory_config(&self) -> RteMemoryConfig {
        unsafe { RteMemoryConfig((*self.0).mem_config) }
    }
}

extern "C" {
    fn _rte_lcore_id() -> u32;
}

/// Initialize the Environment Abstraction Layer (EAL).
pub fn eal_init(args: &Vec<&str>) -> RteResult<usize> {
    let cstrs = args.iter().map(|&s| CString::new(s).unwrap());
    let mut ptrs: Vec<*mut c_char> = cstrs.map(|s| s.as_ptr() as *mut c_char).collect();

    let parsed = unsafe { rte_eal_init(args.len() as i32, ptrs.as_mut_ptr()) };

    if parsed < 0 {
        Err(RteError::Init)
    } else {
        Ok(parsed as usize)
    }
}

/// Function to terminate the application immediately,
/// printing an error message and returning the exit_code back to the shell.
pub fn eal_exit(code: i32, msg: &str) {
    unsafe {
        rte_exit(code, CString::new(msg).unwrap().as_ptr());
    }
}

/// Get the global configuration structure.
pub fn eal_config() -> RteConfig {
    unsafe { RteConfig(rte_eal_get_configuration()) }
}

/// Get the process type in a multi-process setup
pub fn process_type() -> RteProcType {
    unsafe { rte_eal_process_type() }
}

/// Get a lcore's role.
pub fn lcore_role(lcore_id: u32) -> RteLcoreRole {
    unsafe { rte_eal_lcore_role(lcore_id) }
}

/// Check if a primary process is currently alive
pub fn primary_proc_alive() -> bool {
    unsafe { rte_eal_primary_proc_alive(mem::zeroed()) != 0 }
}

/// Whether EAL is using huge pages (disabled by --no-huge option).
pub fn has_hugepages() -> bool {
    unsafe { rte_eal_has_hugepages() != 0 }
}

/// Return the ID of the execution unit we are running on.
pub fn lcore_id() -> u32 {
    unsafe { _rte_lcore_id() }
}

/// Get the id of the master lcore
pub fn master_lcore() -> u32 {
    eal_config().master_lcore()
}

/// Return the number of execution units (lcores) on the system.
pub fn lcore_count() -> usize {
    eal_config().lcore_count()
}

/// Return the ID of the physical socket of the logical core we are running on.
pub fn socket_id() -> u32 {
    unsafe { rte_socket_id() }
}

/// Get the ID of the physical socket of the specified lcore
pub fn lcore_to_socket_id(lcore_id: u32) -> u32 {
    unsafe { lcore_config[lcore_id as usize].socket_id }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ffi::raw::*;

    #[test]
    fn test_eal() {
        assert_eq!(eal_init(&vec![""]).unwrap(), 0);

        assert_eq!(process_type() as u32,
                   Enum_rte_proc_type_t::RTE_PROC_PRIMARY as u32);
    }
}
