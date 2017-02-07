
use ffi;

pub type Addr = ffi::rte_pci_addr;
pub type Id = ffi::rte_pci_id;

pub type RawPciDevice = ffi::pci_device_list_rte_pci_device;
pub type RawPciDevicePtr = *mut RawPciDevice;
