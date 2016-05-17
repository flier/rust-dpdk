
use ffi;

pub type Addr = ffi::Struct_rte_pci_addr;
pub type Id = ffi::Struct_rte_pci_id;

pub type RawDevicePtr = *mut ffi::Struct_rte_pci_device;
