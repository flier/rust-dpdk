use std::mem;
use std::ptr;
use std::os::unix::io::AsRawFd;
use std::os::raw::c_void;

use cfile::{Stream, CFile};

use ffi;

/// This function allocates memory from the huge-page area of memory.
///
/// The memory is not cleared. In NUMA systems, the memory allocated
/// resides on the same NUMA socket as the core that calls this function.
///
pub fn malloc(tag: &'static str, size: usize, align: usize) -> *mut c_void {
    unsafe { ffi::rte_malloc(tag.as_ptr() as *const i8, size as u64, align as u32) }
}

/// Allocate zero'ed memory from the heap.
///
/// Equivalent to rte_malloc() except that the memory zone is initialised with zeros.
/// In NUMA systems, the memory allocated resides on the same NUMA socket as the core that calls this function.
///
pub fn zmalloc(tag: &'static str, size: usize, align: usize) -> *mut c_void {
    unsafe { ffi::rte_zmalloc(tag.as_ptr() as *const i8, size as u64, align as u32) }
}

/// Replacement function for calloc(), using huge-page memory.
///
/// Memory area is initialised with zeros. In NUMA systems,
/// the memory allocated resides on the same NUMA socket as the core that calls this function.
///
pub fn calloc(tag: &'static str, num: usize, size: usize, align: usize) -> *mut c_void {
    unsafe {
        ffi::rte_calloc(tag.as_ptr() as *const i8,
                        num as u64,
                        size as u64,
                        align as u32)
    }
}

/// Replacement function for realloc(), using huge-page memory.
///
/// Reserved area memory is resized, preserving contents.
/// In NUMA systems, the new area resides on the same NUMA socket as the old area.
///
pub fn realloc(ptr: *mut c_void, size: usize, align: usize) -> *mut c_void {
    unsafe { ffi::rte_realloc(ptr, size as u64, align as u32) }
}

/// This function allocates memory from the huge-page area of memory.
///
/// The memory is not cleared.
///
pub fn malloc_socket(tag: &'static str, size: usize, align: usize, socket_id: i32) -> *mut c_void {
    unsafe {
        ffi::rte_malloc_socket(tag.as_ptr() as *const i8,
                               size as u64,
                               align as u32,
                               socket_id)
    }
}

/// Allocate zero'ed memory from the heap.
///
/// Equivalent to rte_malloc() except that the memory zone is initialised with zeros.
///
pub fn zmalloc_socket(tag: &'static str, size: usize, align: usize, socket_id: i32) -> *mut c_void {
    unsafe {
        ffi::rte_zmalloc_socket(tag.as_ptr() as *const i8,
                                size as u64,
                                align as u32,
                                socket_id)
    }
}

/// Replacement function for calloc(), using huge-page memory.
///
/// Memory area is initialised with zeros.
///
pub fn calloc_socket(tag: &'static str,
                     num: usize,
                     size: usize,
                     align: usize,
                     socket_id: i32)
                     -> *mut c_void {
    unsafe {
        ffi::rte_calloc_socket(tag.as_ptr() as *const i8,
                               num as u64,
                               size as u64,
                               align as u32,
                               socket_id)
    }
}

/// Frees the memory space pointed to by the provided pointer.
pub fn free(ptr: *mut c_void) {
    unsafe { ffi::rte_free(ptr as *mut c_void) }
}

/// Get heap statistics for the specified heap.
pub fn get_socket_stats(socket_id: i32) -> Option<ffi::Struct_rte_malloc_socket_stats> {
    unsafe {
        let mut stats: ffi::Struct_rte_malloc_socket_stats = mem::zeroed();

        if ffi::rte_malloc_get_socket_stats(socket_id, &mut stats) == 0 {
            Some(stats)
        } else {
            None
        }
    }
}

/// Dump statistics.
pub fn dump_stats<S: AsRawFd>(s: &S, tag: Option<&str>) {
    if let Ok(f) = CFile::open_stream(s, "w") {
        unsafe {
            ffi::rte_malloc_dump_stats(f.stream() as *mut ffi::FILE,
                                       tag.map_or_else(|| ptr::null(),
                                                       |s| s.as_ptr() as *const i8));
        }
    }
}
