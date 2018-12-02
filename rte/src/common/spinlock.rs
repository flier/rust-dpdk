use std::ops::{Deref, DerefMut};

use libc;

use ffi;

pub type RawSpinLock = ffi::rte_spinlock_t;
pub type RawSpinLockPtr = *mut ffi::rte_spinlock_t;

pub type RawRecursiveSpinLock = ffi::rte_spinlock_recursive_t;
pub type RawRecursiveSpinLockPtr = *mut ffi::rte_spinlock_recursive_t;

pub trait LockImpl {
    type RawLock: ?Sized;

    fn init(p: *mut Self::RawLock);

    fn lock(p: *mut Self::RawLock);

    fn unlock(p: *mut Self::RawLock);

    fn trylock(p: *mut Self::RawLock) -> libc::c_int;

    fn is_locked(p: *mut Self::RawLock) -> libc::c_int;
}

pub struct Lock<T: LockImpl>(T::RawLock);

pub struct LockGuard<'a, T: LockImpl + 'a>(&'a mut Lock<T>);

impl<'a, T: LockImpl> Drop for LockGuard<'a, T> {
    fn drop(&mut self) {
        self.0.unlock();
    }
}

impl<T: LockImpl> Deref for Lock<T> {
    type Target = T::RawLock;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: LockImpl> DerefMut for Lock<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Take the spinlock.
impl<'a, T: LockImpl> Lock<T> {
    #[inline]
    pub fn as_raw(&mut self) -> *mut T::RawLock {
        &mut self.0
    }

    /// Initialize the spinlock to an unlocked state.
    #[inline]
    pub fn init(&mut self) -> &Self {
        T::init(&mut self.0);

        self
    }

    /// Test if the lock is taken.
    #[inline]
    pub fn is_locked(&mut self) -> bool {
        T::is_locked(&mut self.0) != 0
    }

    /// Take the spinlock.
    #[inline]
    pub fn lock(&'a mut self) -> LockGuard<'a, T> {
        T::lock(&mut self.0);

        LockGuard(self)
    }

    /// Try to take the lock.
    #[inline]
    pub fn trylock(&'a mut self) -> Option<LockGuard<'a, T>> {
        if T::trylock(&mut self.0) == 0 {
            None
        } else {
            Some(LockGuard(self))
        }
    }

    /// Release the spinlock.
    #[inline]
    pub fn unlock(&mut self) -> &Self {
        T::unlock(&mut self.0);

        self
    }
}

pub enum SpinLockImpl {}

pub type SpinLock = Lock<SpinLockImpl>;

impl LockImpl for SpinLockImpl {
    type RawLock = RawSpinLock;

    #[inline]
    fn init(p: *mut Self::RawLock) {
        unsafe {
            (*p).locked = 0;
        }
    }

    #[inline]
    fn is_locked(p: *mut Self::RawLock) -> libc::c_int {
        unsafe { (*p).locked }
    }

    #[inline]
    fn lock(p: *mut Self::RawLock) {
        unsafe { _rte_spinlock_lock(p) }
    }

    #[inline]
    fn unlock(p: *mut Self::RawLock) {
        unsafe { _rte_spinlock_unlock(p) }
    }

    #[inline]
    fn trylock(p: *mut Self::RawLock) -> libc::c_int {
        unsafe { _rte_spinlock_trylock(p) }
    }
}

pub enum TmSpinLockImpl {}

pub type TmSpinLock = Lock<TmSpinLockImpl>;

impl LockImpl for TmSpinLockImpl {
    type RawLock = RawSpinLock;

    #[inline]
    fn init(p: *mut Self::RawLock) {
        unsafe {
            (*p).locked = 0;
        }
    }

    #[inline]
    fn is_locked(p: *mut Self::RawLock) -> libc::c_int {
        unsafe { (*p).locked }
    }

    #[inline]
    fn lock(p: *mut Self::RawLock) {
        unsafe { _rte_spinlock_lock_tm(p) }
    }

    #[inline]
    fn unlock(p: *mut Self::RawLock) {
        unsafe { _rte_spinlock_unlock_tm(p) }
    }

    #[inline]
    fn trylock(p: *mut Self::RawLock) -> libc::c_int {
        unsafe { _rte_spinlock_trylock_tm(p) }
    }
}

pub fn tm_supported() -> bool {
    unsafe { _rte_tm_supported() != 0 }
}

pub enum RecursiveSpinLockImpl {}

pub type RecursiveSpinLock = Lock<RecursiveSpinLockImpl>;

impl LockImpl for RecursiveSpinLockImpl {
    type RawLock = RawRecursiveSpinLock;

    #[inline]
    fn init(p: *mut Self::RawLock) {
        unsafe {
            (*p).sl.locked = 0;
            (*p).user = -1;
            (*p).count = 0;
        }
    }

    #[inline]
    fn is_locked(p: *mut Self::RawLock) -> libc::c_int {
        unsafe { (*p).sl.locked }
    }

    #[inline]
    fn lock(p: *mut Self::RawLock) {
        unsafe { _rte_spinlock_recursive_lock(p) }
    }

    #[inline]
    fn unlock(p: *mut Self::RawLock) {
        unsafe { _rte_spinlock_recursive_unlock(p) }
    }

    #[inline]
    fn trylock(p: *mut Self::RawLock) -> libc::c_int {
        unsafe { _rte_spinlock_recursive_trylock(p) }
    }
}

pub enum RecursiveTmSpinLockImpl {}

pub type RecursiveTmSpinLock = Lock<RecursiveTmSpinLockImpl>;

impl LockImpl for RecursiveTmSpinLockImpl {
    type RawLock = RawRecursiveSpinLock;

    #[inline]
    fn init(p: *mut Self::RawLock) {
        unsafe {
            (*p).sl.locked = 0;
            (*p).user = -1;
            (*p).count = 0;
        }
    }

    #[inline]
    fn is_locked(p: *mut Self::RawLock) -> libc::c_int {
        unsafe { (*p).sl.locked }
    }

    #[inline]
    fn lock(p: *mut Self::RawLock) {
        unsafe { _rte_spinlock_recursive_lock_tm(p) }
    }

    #[inline]
    fn unlock(p: *mut Self::RawLock) {
        unsafe { _rte_spinlock_recursive_unlock_tm(p) }
    }

    #[inline]
    fn trylock(p: *mut Self::RawLock) -> libc::c_int {
        unsafe { _rte_spinlock_recursive_trylock_tm(p) }
    }
}

extern "C" {
    fn _rte_spinlock_lock(sl: RawSpinLockPtr);

    fn _rte_spinlock_unlock(sl: RawSpinLockPtr);

    fn _rte_spinlock_trylock(sl: RawSpinLockPtr) -> libc::c_int;

    fn _rte_tm_supported() -> libc::c_int;

    fn _rte_spinlock_lock_tm(sl: RawSpinLockPtr);

    fn _rte_spinlock_unlock_tm(sl: RawSpinLockPtr);

    fn _rte_spinlock_trylock_tm(sl: RawSpinLockPtr) -> libc::c_int;

    fn _rte_spinlock_recursive_lock(sl: RawRecursiveSpinLockPtr);

    fn _rte_spinlock_recursive_unlock(sl: RawRecursiveSpinLockPtr);

    fn _rte_spinlock_recursive_trylock(sl: RawRecursiveSpinLockPtr) -> libc::c_int;

    fn _rte_spinlock_recursive_lock_tm(sl: RawRecursiveSpinLockPtr);

    fn _rte_spinlock_recursive_unlock_tm(sl: RawRecursiveSpinLockPtr);

    fn _rte_spinlock_recursive_trylock_tm(sl: RawRecursiveSpinLockPtr) -> libc::c_int;
}
