use std::ffi::CStr;
use std::mem;
use std::os::raw::{c_uint, c_void};
use std::os::unix::io::AsRawFd;
use std::ptr;

use cfile;
use ffi;
use libc;

use errors::{AsResult, Result};
use memory::SocketId;
use utils::{AsCString, AsRaw, FromRaw};

pub use ffi::{RTE_MEMPOOL_HEADER_COOKIE1, RTE_MEMPOOL_HEADER_COOKIE2, RTE_MEMPOOL_TRAILER_COOKIE};

bitflags! {
    pub struct MemoryPoolFlags: u32 {
        /// Do not spread in memory.
        const MEMPOOL_F_NO_SPREAD       = ffi::MEMPOOL_F_NO_SPREAD;
        /// Do not align objs on cache lines.
        const MEMPOOL_F_NO_CACHE_ALIGN  = ffi::MEMPOOL_F_NO_CACHE_ALIGN;
        /// Default put is "single-producer".
        const MEMPOOL_F_SP_PUT          = ffi::MEMPOOL_F_SP_PUT;
        /// Default get is "single-consumer".
        const MEMPOOL_F_SC_GET          = ffi::MEMPOOL_F_SC_GET;
        /// Internal: pool is created.
        const MEMPOOL_F_POOL_CREATED    = ffi::MEMPOOL_F_POOL_CREATED;
        /// Don't need IOVA contiguous objs.
        const MEMPOOL_F_NO_IOVA_CONTIG  = ffi::MEMPOOL_F_NO_IOVA_CONTIG;
    }
}

/// A mempool constructor callback function.
pub type MemoryPoolConstructor<T> = fn(pool: &MemoryPool, arg: Option<T>);

/// A mempool walk callback function.
pub type MemoryPoolWalkCallback<T> = fn(pool: &MemoryPool, arg: Option<T>);

/// A mempool object iterator callback function.
pub type MemoryPoolObjectCallback<T, O> = fn(pool: &MemoryPool, arg: Option<T>, obj: &mut O, idx: usize);

pub type RawMemoryPool = ffi::rte_mempool;
pub type RawMemoryPoolPtr = *mut ffi::rte_mempool;

/// RTE Mempool.
///
/// A memory pool is an allocator of fixed-size object. It is identified by its name,
/// and uses a ring to store free objects. It provides some other optional services,
/// like a per-core object cache, and an alignment helper to ensure
/// that objects are padded to spread them equally on all RAM channels, ranks, and so on.
///
raw!(pub MemoryPool(RawMemoryPool));

impl Drop for MemoryPool {
    fn drop(&mut self) {
        unsafe { ffi::rte_mempool_free(self.as_raw()) }
    }
}

impl MemoryPool {
    pub fn name(&self) -> &str {
        unsafe { CStr::from_ptr((&self.name[..]).as_ptr()).to_str().unwrap() }
    }

    /// Return the number of entries in the mempool.
    ///
    /// When cache is enabled, this function has to browse the length of
    /// all lcores, so it should not be used in a data path, but only for
    /// debug purposes. User-owned mempool caches are not accounted for.
    pub fn avail_count(&self) -> usize {
        unsafe { ffi::rte_mempool_avail_count(self.as_raw()) as usize }
    }

    /// Return the number of elements which have been allocated from the mempool
    ///
    /// When cache is enabled, this function has to browse the length of
    /// all lcores, so it should not be used in a data path, but only for
    /// debug purposes.
    pub fn in_use_count(&self) -> usize {
        unsafe { ffi::rte_mempool_in_use_count(self.as_raw()) as usize }
    }

    /// Test if the mempool is full.
    pub fn is_full(&self) -> bool {
        self.avail_count() == self.size as usize
    }

    /// Test if the mempool is empty.
    pub fn is_empty(&self) -> bool {
        self.avail_count() == 0
    }

    /// Check the consistency of mempool objects.
    ///
    /// Verify the coherency of fields in the mempool structure.
    /// Also check that the cookies of mempool objects (even the ones that are not present in pool)
    /// have a correct value. If not, a panic will occur.
    ///
    pub fn audit(&self) {
        unsafe { ffi::rte_mempool_audit(self.as_raw()) }
    }

    /// Dump the status of the mempool to the console.
    pub fn dump<S: AsRawFd>(&self, s: &S) -> Result<()> {
        let f = cfile::open_stream(s, "w")?;

        unsafe { ffi::rte_mempool_dump(f.stream() as *mut ffi::FILE, self.as_raw()) };

        Ok(())
    }

    /// Call a function for each mempool object in a memory chunk
    ///
    /// Iterate across objects of the given size and alignment in the provided chunk of memory.
    /// The given memory buffer can consist of disjointed physical pages.
    ///
    /// For each object, call the provided callback (if any).
    /// This function is used to populate a mempool, or walk through all the elements of a mempool,
    /// or estimate how many elements of the given size could be created in the given memory buffer.
    ///
    pub fn walk<T, O>(&mut self, callback: MemoryPoolObjectCallback<T, O>, arg: Option<T>) -> usize {
        unsafe {
            ffi::rte_mempool_obj_iter(
                self.as_raw(),
                Some(obj_cb_stub::<T, O>),
                Box::into_raw(Box::new(MemoryPoolObjectCallbackContext { callback, arg })) as *mut _,
            ) as usize
        }
    }
}

/// Create a new mempool named name in memory.
///
/// This function uses memzone_reserve() to allocate memory.
/// The pool contains n elements of elt_size. Its size is set to n.
/// All elements of the mempool are allocated together with the mempool header,
/// in one physically continuous chunk of memory.
///
pub fn create<S, M, T, O>(
    name: S,
    n: u32,
    cache_size: u32,
    private_data_size: u32,
    mp_init: Option<MemoryPoolConstructor<M>>,
    mp_init_arg: Option<M>,
    obj_init: Option<MemoryPoolObjectCallback<T, O>>,
    obj_init_arg: Option<T>,
    socket_id: SocketId,
    flags: MemoryPoolFlags,
) -> Result<MemoryPool>
where
    S: AsRef<str>,
{
    let name = name.as_cstring();

    let mp_init_ctx = if let Some(callback) = mp_init {
        Box::into_raw(Box::new(MemoryPoolConstructorContext::<M> {
            callback,
            arg: mp_init_arg,
        })) as *mut c_void
    } else {
        ptr::null_mut()
    };
    let obj_init_ctx = if let Some(callback) = obj_init {
        Box::into_raw(Box::new(MemoryPoolObjectCallbackContext::<T, O> {
            callback,
            arg: obj_init_arg,
        })) as *mut c_void
    } else {
        ptr::null_mut()
    };

    unsafe {
        ffi::rte_mempool_create(
            name.as_ptr(),
            n,
            mem::size_of::<O>() as u32,
            cache_size,
            private_data_size,
            if mp_init.is_none() {
                None
            } else {
                Some(mp_init_stub::<T>)
            },
            mp_init_ctx,
            if obj_init.is_none() {
                None
            } else {
                Some(obj_cb_stub::<T, O>)
            },
            obj_init_ctx,
            socket_id,
            flags.bits,
        )
    }.as_result()
    .map(MemoryPool)
}

/// Create an empty mempool
///
/// The mempool is allocated and initialized, but it is not populated:
/// no memory is allocated for the mempool elements.
/// The user has to call rte_mempool_populate_*() to add memory chunks to the pool.
/// Once populated, the user may also want to initialize each object with rte_mempool_obj_iter().
pub fn create_empty<S, O>(
    name: S,
    n: u32,
    cache_size: u32,
    private_data_size: u32,
    socket_id: SocketId,
    flags: MemoryPoolFlags,
) -> Result<MemoryPool>
where
    S: AsRef<str>,
{
    let name = name.as_cstring();

    unsafe {
        ffi::rte_mempool_create_empty(
            name.as_ptr(),
            n,
            mem::size_of::<O>() as u32,
            cache_size,
            private_data_size,
            socket_id,
            flags.bits,
        )
    }.as_result()
    .map(MemoryPool)
}

struct MemoryPoolConstructorContext<T> {
    callback: MemoryPoolConstructor<T>,
    arg: Option<T>,
}

unsafe extern "C" fn mp_init_stub<T>(mp: *mut ffi::rte_mempool, arg: *mut c_void) {
    let mp = MemoryPool::from(mp);
    let ctx = Box::from_raw(arg as *mut MemoryPoolConstructorContext<T>);

    (ctx.callback)(&mp, ctx.arg);

    mem::forget(mp);
}

struct MemoryPoolObjectCallbackContext<T, O> {
    callback: MemoryPoolObjectCallback<T, O>,
    arg: Option<T>,
}

unsafe extern "C" fn obj_cb_stub<T, O>(mp: *mut ffi::rte_mempool, arg: *mut c_void, obj: *mut c_void, obj_idx: c_uint) {
    let mp = MemoryPool::from(mp);
    let ctx = Box::from_raw(arg as *mut MemoryPoolObjectCallbackContext<T, O>);

    (ctx.callback)(&mp, ctx.arg, (obj as *mut O).as_mut().unwrap(), obj_idx as usize);

    mem::forget(mp);
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
pub fn walk<T>(callback: MemoryPoolWalkCallback<T>, arg: Option<T>) {
    let ctx = Box::into_raw(Box::new(PoolWalkContext { callback, arg })) as *mut _;

    unsafe {
        ffi::rte_mempool_walk(Some(pool_walk_stub::<T>), ctx);
    }
}

struct PoolWalkContext<T> {
    callback: MemoryPoolWalkCallback<T>,
    arg: Option<T>,
}

unsafe extern "C" fn pool_walk_stub<T>(mp: *mut ffi::rte_mempool, ctxt: *mut libc::c_void) {
    let mp = MemoryPool::from(mp);
    let ctxt = Box::from_raw(ctxt as *mut PoolWalkContext<T>);

    (ctxt.callback)(&mp, ctxt.arg);

    mem::forget(mp)
}
