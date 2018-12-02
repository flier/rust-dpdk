use std::ffi::CString;

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
