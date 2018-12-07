use std::ffi::CString;
use std::ops::Deref;

pub trait AsRaw {
    type Raw;

    fn as_raw(&self) -> *mut Self::Raw;
}

pub trait IntoRaw: AsRaw {
    fn into_raw(self) -> *mut Self::Raw;
}

pub trait FromRaw: AsRaw
where
    Self: Sized,
{
    fn from_raw(raw: *mut Self::Raw) -> Option<Self>;
}

pub trait AsCString {
    fn as_cstring(&self) -> CString;
}

impl<T> AsCString for T
where
    T: AsRef<str>,
{
    fn as_cstring(&self) -> CString {
        let mut v = self.as_ref().as_bytes().to_owned();
        v.push(0);
        unsafe { CString::from_vec_unchecked(v) }
    }
}
