use std::mem;
use std::ops::Deref;
use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use std::os::unix::io::AsRawFd;

use ffi;

use errors::{Error, Result};
use cfile::{Stream, CFile};

bitflags! {
    pub flags MemoryPoolFlags: u32 {
        /// Do not spread in memory.
        const MEMPOOL_F_NO_SPREAD       = 0x0001,
        /// Do not align objs on cache lines.
        const MEMPOOL_F_NO_CACHE_ALIGN  = 0x0002,
        /// Default put is "single-producer".
        const MEMPOOL_F_SP_PUT          = 0x0004,
        /// Default get is "single-consumer".
        const MEMPOOL_F_SC_GET          = 0x0008,
    }
}

pub type RawMemoryPoolPtr = *mut ffi::Struct_rte_mempool;

/// A mempool constructor callback function.
pub type MemoryPoolConstructor<T> = fn(pool: RawMemoryPoolPtr, arg: Option<&mut T>);

/// An object constructor callback function for mempool.
pub type MemoryPoolObjectContructor<T> = fn(pool: RawMemoryPoolPtr,
                                            arg: Option<&mut T>,
                                            elt: *mut c_void,
                                            u32);

/// A mempool walk callback function.
pub type MemoryPoolWalkCallback<T> = fn(pool: RawMemoryPoolPtr, arg: Option<&mut T>);

/// A mempool object iterator callback function.
pub type MemoryPoolObjectIterator<T, P> = fn(arg: Option<&mut T>,
                                             obj_start: *mut P,
                                             obj_end: *mut P,
                                             obj_index: u32);

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
    fn phys_addr(&self) -> ffi::phys_addr_t;

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

    /// Virtual address of the first mempool object.
    fn elt_va_start(&self) -> ffi::uintptr_t;

    // Virtual address of the <size + 1> mempool object.
    fn elt_va_end(&self) -> ffi::uintptr_t;

    /// Array of physical page addresses for the mempool objects buffer.
    fn elt_pa(&self) -> &[ffi::phys_addr_t];
}

pub trait MemoryPoolDebug: MemoryPool {
    /// Return the number of entries in the mempool.
    ///
    /// When cache is enabled, this function has to browse the length of all lcores,
    /// so it should not be used in a data path, but only for debug purposes.
    ///
    fn count(&self) -> u32;

    /// Return the number of free entries in the mempool ring.
    ///
    /// i.e. how many entries can be freed back to the mempool.
    ///
    fn free_count(&self) -> u32 {
        self.size() - self.count()
    }

    /// Test if the mempool is full.
    fn full(&self) -> bool {
        self.size() == self.count()
    }

    /// Test if the mempool is empty.
    fn empty(&self) -> bool {
        self.count() == 0
    }

    /// Check the consistency of mempool objects.
    ///
    /// Verify the coherency of fields in the mempool structure.
    /// Also check that the cookies of mempool objects (even the ones that are not present in pool)
    /// have a correct value. If not, a panic will occur.
    ///
    fn audit(&self);

    /// Dump the status of the mempool to the console.
    fn dump<S: AsRawFd>(&self, s: &S);

    /// Call a function for each mempool object in a memory chunk
    ///
    /// Iterate across objects of the given size and alignment in the provided chunk of memory.
    /// The given memory buffer can consist of disjointed physical pages.
    ///
    /// For each object, call the provided callback (if any).
    /// This function is used to populate a mempool, or walk through all the elements of a mempool,
    /// or estimate how many elements of the given size could be created in the given memory buffer.
    ///
    fn walk<T, P>(&self,
                  elt_num: u32,
                  obj_iter: Option<MemoryPoolObjectIterator<T, P>>,
                  obj_iter_arg: Option<&T>)
                  -> u32;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RawMemoryPool(RawMemoryPoolPtr);

impl RawMemoryPool {
    /// Create a new mempool named name in memory.
    ///
    /// This function uses memzone_reserve() to allocate memory.
    /// The pool contains n elements of elt_size. Its size is set to n.
    /// All elements of the mempool are allocated together with the mempool header,
    /// in one physically continuous chunk of memory.
    ///
    pub fn create<T, O>(name: &str,
                        n: u32,
                        elt_size: u32,
                        cache_size: u32,
                        private_data_size: u32,
                        mp_init: Option<MemoryPoolConstructor<T>>,
                        mp_init_arg: Option<&T>,
                        obj_init: Option<MemoryPoolObjectContructor<O>>,
                        obj_init_arg: Option<&O>,
                        socket_id: i32,
                        flags: MemoryPoolFlags)
                        -> Result<RawMemoryPool> {
        let name = try!(CString::new(name))
                       .as_bytes_with_nul()
                       .as_ptr() as *const i8;


        let p = unsafe {
            ffi::rte_mempool_create(name,
                                    n,
                                    elt_size,
                                    cache_size,
                                    private_data_size,
                                    mem::transmute(mp_init),
                                    mem::transmute(mp_init_arg),
                                    mem::transmute(obj_init),
                                    mem::transmute(obj_init_arg),
                                    socket_id,
                                    flags.bits)
        };

        if p.is_null() {
            Err(Error::rte_error())
        } else {
            Ok(RawMemoryPool(p))
        }
    }

    pub fn from_raw(p: RawMemoryPoolPtr) -> RawMemoryPool {
        RawMemoryPool(p)
    }

    pub fn lookup(name: &str) -> Option<RawMemoryPool> {
        let p = unsafe {
            ffi::rte_mempool_lookup(CString::new(name)
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
        if let Ok(f) = CFile::open_stream(s, "w") {
            unsafe {
                ffi::rte_mempool_list_dump(f.stream() as *mut ffi::FILE);
            }
        }
    }

    /// Walk list of all memory pools
    pub fn walk<T>(callback: Option<MemoryPoolWalkCallback<T>>, arg: Option<&T>) {
        unsafe {
            ffi::rte_mempool_walk(mem::transmute(callback), mem::transmute(arg));
        }
    }
}

impl Deref for RawMemoryPool {
    type Target = RawMemoryPoolPtr;

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
    fn phys_addr(&self) -> ffi::phys_addr_t {
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

    #[inline]
    fn elt_va_start(&self) -> ffi::uintptr_t {
        unsafe { (*self.0).elt_va_start }
    }

    #[inline]
    fn elt_va_end(&self) -> ffi::uintptr_t {
        unsafe { (*self.0).elt_va_end }
    }

    #[inline]
    fn elt_pa(&self) -> &[ffi::phys_addr_t] {
        unsafe { &(*self.0).elt_pa[..(*self.0).pg_num as usize] }
    }
}

impl MemoryPoolDebug for RawMemoryPool {
    fn count(&self) -> u32 {
        unsafe { ffi::rte_mempool_count(self.0) }
    }

    fn audit(&self) {
        unsafe { ffi::rte_mempool_audit(self.0) }
    }

    fn dump<S: AsRawFd>(&self, s: &S) {
        if let Ok(f) = CFile::open_stream(s, "w") {
            unsafe {
                ffi::rte_mempool_dump(f.stream() as *mut ffi::FILE, self.0);
            }
        }
    }

    fn walk<T, P>(&self,
                  elt_num: u32,
                  obj_iter: Option<MemoryPoolObjectIterator<T, P>>,
                  obj_iter_arg: Option<&T>)
                  -> u32 {
        unsafe {
            let p = *self.0;
            let elt_sz = (p.header_size + p.elt_size + p.trailer_size) as ffi::size_t;

            ffi::rte_mempool_obj_iter(mem::transmute(p.elt_va_start),
                                      elt_num,
                                      elt_sz,
                                      1,
                                      p.elt_pa.as_ptr(),
                                      p.pg_num,
                                      p.pg_shift,
                                      mem::transmute(obj_iter),
                                      mem::transmute(obj_iter_arg))
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate env_logger;

    use std::mem;
    use std::os::raw::c_void;

    use log::LogLevel::Debug;
    use cfile::CFile;

    use ffi;

    use super::*;
    use super::super::eal;

    #[test]
    fn test_mempool() {
        let _ = env_logger::init();

        assert!(eal::init(&vec![String::from("test")]));

        let p = RawMemoryPool::create::<c_void, c_void>("test", // name
                                      16, // nll
                                      128, // elt_size
                                      0, // cache_size
                                      32, // private_data_size
                                      None, // mp_init
                                      None, // mp_init_arg
                                      None, // obj_init
                                      None, // obj_init_arg
                                      ffi::SOCKET_ID_ANY, // socket_id
                                      MEMPOOL_F_SP_PUT | MEMPOOL_F_SC_GET) // flags
                    .unwrap();

        assert_eq!(p.name(), "test");
        assert_eq!(p.size(), 16);
        assert!(p.phys_addr() != 0);
        assert_eq!(p.cache_size(), 0);
        assert_eq!(p.cache_flushthresh(), 0);
        assert_eq!(p.elt_size(), 128);
        assert_eq!(p.header_size(), 64);
        assert_eq!(p.trailer_size(), 0);
        assert_eq!(p.private_data_size(), 64);
        assert_eq!((p.elt_va_end() - p.elt_va_start()) as u32,
                   (p.header_size() + p.elt_size()) * p.size());
        assert_eq!(p.elt_pa().len(), 1);

        assert_eq!(p.count(), 16);
        assert_eq!(p.free_count(), 0);
        assert!(p.full());
        assert!(!p.empty());

        p.audit();

        if log_enabled!(Debug) {
            let stdout = CFile::open_stdout().unwrap();

            p.dump(&stdout);
        }

        let mut elements: Vec<(u32, usize)> = Vec::new();

        fn walk_element(elements: Option<&mut Vec<(u32, usize)>>,
                        obj_start: *mut c_void,
                        obj_end: *mut c_void,
                        obj_index: u32) {
            unsafe {
                let obj_addr: usize = mem::transmute(obj_start);
                let obj_end: usize = mem::transmute(obj_end);

                elements.unwrap()
                        .push((obj_index, obj_end - obj_addr));
            }
        }

        assert_eq!(p.walk(4, Some(walk_element), Some(&mut elements)), 4);

        assert_eq!(elements.len(), 4);

        assert_eq!(p, RawMemoryPool::lookup("test").unwrap());

        let mut pools: Vec<RawMemoryPoolPtr> = Vec::new();

        fn walk_mempool(pool: RawMemoryPoolPtr, pools: Option<&mut Vec<RawMemoryPoolPtr>>) {
            pools.unwrap().push(pool);
        }

        RawMemoryPool::walk(Some(walk_mempool), Some(&mut pools));

        assert!(pools.iter().find(|pool| **pool == *p).is_some());

        if log_enabled!(Debug) {
            let stdout = CFile::open_stdout().unwrap();

            RawMemoryPool::list_dump(&stdout);
        }
    }
}
