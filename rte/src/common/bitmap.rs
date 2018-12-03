use errors::{AsResult, Result};
use ffi;

pub type Position = u32;
pub type Slab = u64;

pub type RawBitmap = ffi::rte_bitmap;
pub type RawBitmapPtr = *mut ffi::rte_bitmap;

///  RTE Bitmap
///
///  The bitmap component provides a mechanism to manage large arrays of bits
///  through bit get/set/clear and bit array scan operations.
///
///  The bitmap scan operation is optimized for 64-bit CPUs using 64/128 byte cache
///  lines. The bitmap is hierarchically organized using two arrays (array1 and
///  array2), with each bit in array1 being associated with a full cache line
///  (512/1024 bits) of bitmap bits, which are stored in array2: the bit in array1
///  is set only when there is at least one bit set within its associated array2
///  bits, otherwise the bit in array1 is cleared. The read and write operations
///  for array1 and array2 are always done in slabs of 64 bits.
///
///  This bitmap is not thread safe. For lock free operation on a specific bitmap
///  instance, a single writer thread performing bit set/clear operations is
///  allowed, only the writer thread can do bitmap scan operations, while there
///  can be several reader threads performing bit get operations in parallel with
///  the writer thread. When the use of locking primitives is acceptable, the
///  serialization of the bit set/clear and bitmap scan operations needs to be
///  enforced by the caller, while the bit get operation does not require locking
///  the bitmap.
#[repr(transparent)]
#[derive(Debug)]
pub struct Bitmap(RawBitmapPtr);

impl Drop for Bitmap {
    fn drop(&mut self) {
        unsafe {
            ffi::rte_bitmap_free(self.0);
        }
    }
}

impl Bitmap {
    /// Bitmap memory footprint calculation
    pub fn memory_footprint(bits: u32) -> u32 {
        unsafe { ffi::rte_bitmap_get_memory_footprint(bits) }
    }

    /// Bitmap initialization
    pub fn init(bits: u32, mem: *mut u8, mem_size: u32) -> Result<Self> {
        unsafe { ffi::rte_bitmap_init(bits, mem, mem_size) }
            .as_result()
            .map(Bitmap)
    }

    /// Bitmap reset
    pub fn reset(&mut self) {
        unsafe { ffi::rte_bitmap_reset(self.0) }
    }

    /// Bitmap location prefetch into CPU L1 cache
    pub fn prefetch0(&self, pos: Position) {
        unsafe { ffi::rte_bitmap_prefetch0(self.0, pos) }
    }

    /// Bitmap bit get
    pub fn get(&self, pos: Position) -> bool {
        unsafe { ffi::rte_bitmap_get(self.0, pos) != 0 }
    }

    /// Bitmap bit set
    pub fn set(&mut self, pos: Position) {
        unsafe { ffi::rte_bitmap_set(self.0, pos) }
    }

    /// Bitmap slab set
    pub fn set_slab(&mut self, pos: Position, slab: Slab) {
        unsafe { ffi::rte_bitmap_set_slab(self.0, pos, slab) }
    }

    /// Bitmap bit clear
    pub fn clear(&mut self, pos: Position) {
        unsafe { ffi::rte_bitmap_clear(self.0, pos) }
    }

    /// Bitmap scan (with automatic wrap-around)
    pub fn scan(&self) -> Option<(Position, Slab)> {
        let mut pos = 0;
        let mut slab = 0;

        if unsafe { ffi::rte_bitmap_scan(self.0, &mut pos, &mut slab) } == 0 {
            None
        } else {
            Some((pos, slab))
        }
    }
}
