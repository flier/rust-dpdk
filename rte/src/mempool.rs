use std::ops::Deref;
use std::ffi::{CStr, CString};
use std::os::unix::io::AsRawFd;

use libc;

use ffi::raw::*;

/// RTE Mempool.
///
/// A memory pool is an allocator of fixed-size object. It is identified by its name,
/// and uses a ring to store free objects. It provides some other optional services,
/// like a per-core object cache, and an alignment helper to ensure
/// that objects are padded to spread them equally on all RAM channels, ranks, and so on.
///
pub trait MemoryPool {
    /// Name of mempool.
    fn name(&self) -> &str;

    /// Size of the mempool.
    fn size(&self) -> u32;

    /// Phys. addr. of mempool struct.
    fn phys_addr(&self) -> u64;

    /// Size of per-lcore local cache.
    fn cache_size(&self) -> u32;

    /// Threshold before we flush excess elements.
    fn cache_flushthresh(&self) -> u32;

    /// Size of an element.
    fn elt_size(&self) -> u32;

    /// Size of header (before elt).
    fn header_size(&self) -> u32;

    /// Size of trailer (after elt).
    fn trailer_size(&self) -> u32;

    /// Size of private data.
    fn private_data_size(&self) -> u32;
}

pub trait MemoryPoolDebug {
    /// Return the number of entries in the mempool.
    ///
    /// When cache is enabled, this function has to browse the length of all lcores,
    /// so it should not be used in a data path, but only for debug purposes.
    ///
    fn count(&self) -> usize;

    /// Check the consistency of mempool objects.
    ///
    /// Verify the coherency of fields in the mempool structure.
    /// Also check that the cookies of mempool objects (even the ones that are not present in pool)
    /// have a correct value. If not, a panic will occur.
    ///
    fn audit(&self);

    /// Dump the status of the mempool to the console.
    fn dump<S: AsRawFd>(&self, s: &S);
}

#[derive(Clone, Copy, Debug)]
pub struct RawMemoryPool(*mut Struct_rte_mempool);

impl RawMemoryPool {
    pub fn from_raw(p: *mut Struct_rte_mempool) -> RawMemoryPool {
        RawMemoryPool(p)
    }

    pub fn lookup(name: &str) -> Option<RawMemoryPool> {
        let p = unsafe {
            rte_mempool_lookup(CString::new(name)
                                   .unwrap()
                                   .as_bytes_with_nul()
                                   .as_ptr() as *const i8)
        };

        if p.is_null() {
            None
        } else {
            Some(RawMemoryPool(p))
        }
    }

    /// Dump the status of all mempools on the console
    pub fn list_dump<S: AsRawFd>(s: &S) {
        unsafe {
            let f = libc::fdopen(s.as_raw_fd(), "w".as_ptr() as *const i8);

            rte_mempool_list_dump(f as *mut FILE);

            libc::fclose(f);
        }
    }
}

impl Deref for RawMemoryPool {
    type Target = *mut Struct_rte_mempool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MemoryPool for RawMemoryPool {
    #[inline]
    fn name(&self) -> &str {
        unsafe {
            let name = &((*self.0).name)[..];

            CStr::from_ptr(name.as_ptr()).to_str().unwrap()
        }
    }

    #[inline]
    fn size(&self) -> u32 {
        unsafe { (*self.0).size }
    }

    #[inline]
    fn phys_addr(&self) -> u64 {
        unsafe { (*self.0).phys_addr }
    }

    #[inline]
    fn cache_size(&self) -> u32 {
        unsafe { (*self.0).cache_size }
    }

    #[inline]
    fn cache_flushthresh(&self) -> u32 {
        unsafe { (*self.0).cache_flushthresh }
    }

    #[inline]
    fn elt_size(&self) -> u32 {
        unsafe { (*self.0).elt_size }
    }

    #[inline]
    fn header_size(&self) -> u32 {
        unsafe { (*self.0).header_size }
    }

    #[inline]
    fn trailer_size(&self) -> u32 {
        unsafe { (*self.0).trailer_size }
    }

    #[inline]
    fn private_data_size(&self) -> u32 {
        unsafe { (*self.0).private_data_size }
    }
}

impl MemoryPoolDebug for RawMemoryPool {
    fn count(&self) -> usize {
        unsafe { rte_mempool_count(self.0) as usize }
    }

    fn audit(&self) {
        unsafe { rte_mempool_audit(self.0) }
    }

    fn dump<S: AsRawFd>(&self, s: &S) {
        unsafe {
            let f = libc::fdopen(s.as_raw_fd(), "w".as_ptr() as *const i8);

            rte_mempool_dump(f as *mut FILE, self.0);

            libc::fclose(f);
        }
    }
}
