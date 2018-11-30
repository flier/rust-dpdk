use ffi;

pub type RawDevice = ffi::rte_device;
pub type RawDevicePtr = *mut ffi::rte_device;

pub struct Device(RawDevicePtr);

impl From<RawDevicePtr> for Device {
    fn from(p: RawDevicePtr) -> Self {
        Device(p)
    }
}
