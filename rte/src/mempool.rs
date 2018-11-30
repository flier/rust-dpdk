use std::ffi::CStr;
use std::mem;
use std::os::raw::c_void;
use std::os::unix::io::AsRawFd;
use std::ptr;

use cfile;

use ffi;

use errors::Result;
use libc;
use memory::SocketId;

bitflags! {
    pub struct MemoryPoolFlags: u32 {
        /// Do not spread in memory.
        const MEMPOOL_F_NO_SPREAD       = 0x0001;
        /// Do not align objs on cache lines.
        const MEMPOOL_F_NO_CACHE_ALIGN  = 0x0002;
        /// Default put is "single-producer".
        const MEMPOOL_F_SP_PUT          = 0x0004;
        /// Default get is "single-consumer".
        const MEMPOOL_F_SC_GET          = 0x0008;
    }
}

pub type RawMemoryPool = ffi::rte_mempool;
pub type RawMemoryPoolPtr = *mut ffi::rte_mempool;

/// A mempool constructor callback function.
pub type MemoryPoolConstructor<T> = fn(pool: RawMemoryPoolPtr, arg: Option<&mut T>);

/// An object constructor callback function for mempool.
pub type MemoryPoolObjectContructor<T> =
    fn(pool: RawMemoryPoolPtr, arg: Option<&mut T>, elt: *mut c_void, u32);

/// A mempool walk callback function.
pub type MemoryPoolWalkCallback<T> = fn(pool: RawMemoryPoolPtr, arg: Option<&mut T>);

/// A mempool object iterator callback function.
pub type MemoryPoolObjectCallback<T, P> =
    fn(pool: RawMemoryPoolPtr, opaque: *mut T, obj: *mut P, obj_index: u32);

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
}

pub trait MemoryPoolDebug: MemoryPool {
    /// Return the number of entries in the mempool.
    ///
    /// When cache is enabled, this function has to browse the length of
    /// all lcores, so it should not be used in a data path, but only for
    /// debug purposes. User-owned mempool caches are not accounted for.
    fn avail_count(&self) -> usize;

    /// Return the number of elements which have been allocated from the mempool
    ///
    /// When cache is enabled, this function has to browse the length of
    /// all lcores, so it should not be used in a data path, but only for
    /// debug purposes.
    fn in_use_count(&self) -> usize;

    /// Test if the mempool is full.
    fn is_full(&self) -> bool;

    /// Test if the mempool is empty.
    fn is_empty(&self) -> bool {
        self.avail_count() == 0
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
    fn walk<T, P>(
        &mut self,
        obj_iter: Option<MemoryPoolObjectCallback<T, P>>,
        obj_iter_arg: Option<&mut T>,
    ) -> u32;
}

/// Create a new mempool named name in memory.
///
/// This function uses memzone_reserve() to allocate memory.
/// The pool contains n elements of elt_size. Its size is set to n.
/// All elements of the mempool are allocated together with the mempool header,
/// in one physically continuous chunk of memory.
///
pub fn create<T, O>(
    name: &str,
    n: u32,
    elt_size: u32,
    cache_size: u32,
    private_data_size: u32,
    mp_init: Option<MemoryPoolConstructor<T>>,
    mp_init_arg: Option<&T>,
    obj_init: Option<MemoryPoolObjectContructor<O>>,
    obj_init_arg: Option<&O>,
    socket_id: SocketId,
    flags: MemoryPoolFlags,
) -> Result<RawMemoryPoolPtr> {
    let p = unsafe {
        ffi::rte_mempool_create(
            try!(to_cptr!(name)),
            n,
            elt_size,
            cache_size,
            private_data_size,
            mem::transmute(mp_init),
            mem::transmute(mp_init_arg),
            mem::transmute(obj_init),
            mem::transmute(obj_init_arg),
            socket_id,
            flags.bits,
        )
    };

    rte_check!(p, NonNull)
}

pub fn lookup(name: &str) -> Result<RawMemoryPoolPtr> {
    let p = unsafe { ffi::rte_mempool_lookup(try!(to_cptr!(name))) };

    rte_check!(p, NonNull)
}

/// Dump the status of all mempools on the console
pub fn list_dump<S: AsRawFd>(s: &S) {
    if let Ok(f) = cfile::open_stream(s, "w") {
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

impl MemoryPool for RawMemoryPool {
    #[inline]
    fn name(&self) -> &str {
        unsafe {
            let name = &(self.name)[..];

            CStr::from_ptr(name.as_ptr()).to_str().unwrap()
        }
    }
}

impl MemoryPoolDebug for RawMemoryPool {
    fn avail_count(&self) -> usize {
        unsafe { ffi::rte_mempool_avail_count(self) as usize }
    }

    fn in_use_count(&self) -> usize {
        unsafe { ffi::rte_mempool_in_use_count(self) as usize }
    }

    fn is_full(&self) -> bool {
        self.avail_count() == self.size as usize
    }

    fn audit(&self) {
        unsafe { ffi::rte_mempool_audit(self as *const _ as *mut _) }
    }

    fn dump<S: AsRawFd>(&self, s: &S) {
        if let Ok(f) = cfile::open_stream(s, "w") {
            unsafe {
                ffi::rte_mempool_dump(f.stream() as *mut ffi::FILE, self as *const _ as *mut _);
            }
        }
    }

    fn walk<T, P>(
        &mut self,
        obj_iter: Option<MemoryPoolObjectCallback<T, P>>,
        obj_iter_arg: Option<&mut T>,
    ) -> u32 {
        unsafe {
            ffi::rte_mempool_obj_iter(
                self,
                mem::transmute(obj_iter),
                if let Some(arg) = obj_iter_arg {
                    arg as *mut _ as *mut libc::c_void
                } else {
                    ptr::null_mut()
                },
            )
        }
    }
}
