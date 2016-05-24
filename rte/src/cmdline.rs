use std::mem;
use std::ptr;
use std::str;
use std::result;
use std::slice;
use std::string;
use std::iter::Iterator;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::Path;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::{c_char, c_void};

use libc;

use ffi;

use errors::{Error, Result};
use ether;

pub type RawTokenHeader = ffi::Struct_cmdline_token_hdr;
pub type RawTokenPtr = *const RawTokenHeader;
pub type RawStrToken = ffi::Struct_cmdline_token_string;
pub type RawNumToken = ffi::Struct_cmdline_token_num;
pub type RawIpAddrToken = ffi::Struct_cmdline_token_ipaddr;
pub type RawEtherAddrToken = ffi::Struct_cmdline_token_etheraddr;
pub type RawPortListToken = ffi::Struct_cmdline_token_portlist;

pub enum Token<T> {
    Raw(RawTokenPtr, PhantomData<T>),
    Str(RawStrToken, PhantomData<T>),
    Num(RawNumToken, PhantomData<T>),
    IpAddr(RawIpAddrToken, PhantomData<T>),
    EtherAddr(RawEtherAddrToken, PhantomData<T>),
    PortList(RawPortListToken, PhantomData<T>),
}

impl<T> Token<T> {
    pub fn as_raw(&self) -> RawTokenPtr {
        match self {
            &Token::Raw(hdr, _) => hdr,
            &Token::Str(ref token, _) => &token.hdr,
            &Token::Num(ref token, _) => &token.hdr,
            &Token::IpAddr(ref token, _) => &token.hdr,
            &Token::EtherAddr(ref token, _) => &token.hdr,
            &Token::PortList(ref token, _) => &token.hdr,
        }
    }
}

impl<T> Drop for Token<T> {
    fn drop(&mut self) {
        if let &mut Token::Str(ref token, _) = self {
            unsafe { libc::free(token.string_data._str as *mut libc::c_void) }
        }
    }
}

pub type FixedStr = ffi::cmdline_fixed_string_t;
pub type IpNetAddr = ffi::cmdline_ipaddr_t;
pub type EtherAddr = ffi::Struct_ether_addr;
pub type PortList = ffi::cmdline_portlist_t;

pub fn str(s: &FixedStr) -> result::Result<&str, str::Utf8Error> {
    unsafe { str::from_utf8(CStr::from_ptr(s.as_ptr()).to_bytes()) }
}

pub fn ipaddr(ip: &mut IpNetAddr) -> IpAddr {
    unsafe {
        if ip.family == libc::AF_INET as u8 {
            IpAddr::V4(Ipv4Addr::from((*ip.addr.ipv4()).s_addr))
        } else {
            let a: &[u16] = slice::from_raw_parts(mem::transmute(ip.addr.ipv6()), 8);

            IpAddr::V6(Ipv6Addr::new(a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7]))
        }
    }
}

pub fn etheraddr(addr: &EtherAddr) -> ether::EtherAddr {
    ether::EtherAddr::from(addr.addr_bytes)
}

pub fn portlist(ports: &PortList) -> Vec<u32> {
    (0..32).filter(|portid| ((1 << portid) as u32 & ports.map) != 0).collect()
}

pub fn is_end_of_token(c: u8) -> bool {
    unsafe { ffi::cmdline_isendoftoken(c as i8) != 0 }
}

pub type RawTokenOps = ffi::Struct_cmdline_token_ops;

macro_rules! cstr {
    ($s:expr) => (
        try!(CString::new($s.to_string())).as_ptr() as *const i8
    )
}

/// Macro to get the offset of a struct field in bytes from the address of the
/// struct.
///
/// This macro is identical to `offset_of!` but doesn't give a warning about
/// unnecessary unsafe blocks when invoked from unsafe code.
#[macro_export]
macro_rules! offset_of_unsafe {
    ($container:path, $field:ident) => {{
        // Make sure the field exists, otherwise this could result in UB if the
        // field is accessed through Deref. This will cause a null dereference
        // at runtime since the offset can't be reduced to a constant.
        let $container { $field : _, .. };

        // Yes, this is technically derefencing a null pointer. However, Rust
        // currently accepts this and reduces it to a constant, even in debug
        // builds!
        &(*(0 as *const $container)).$field as *const _ as isize
    }};
}

/// Macro to get the offset of a struct field in bytes from the address of the
/// struct.
///
/// This macro will cause a warning if it is invoked in an unsafe block. Use the
/// `offset_of_unsafe` macro instead to avoid this warning.
#[macro_export]
macro_rules! offset_of {
    ($container:path, $field:ident) => {
        unsafe { offset_of_unsafe!($container, $field) }
    };
}

#[macro_export]
macro_rules! TOKEN_STRING_INITIALIZER {
    ($container:path, $field:ident) => ({
        $crate::cmdline::Token::Str(
            $crate::raw::Struct_cmdline_token_string {
                hdr: $crate::raw::Struct_cmdline_token_hdr {
                    ops: unsafe { &mut $crate::raw::cmdline_token_string_ops },
                    offset: offset_of!($container, $field) as u32,
                },
                string_data: $crate::raw::Struct_cmdline_token_string_data {
                    _str: ::std::ptr::null(),
                },
            }, ::std::marker::PhantomData
        )
    });

    ($container:path, $field:ident, $string:expr) => ({
        let p = unsafe { ::libc::calloc(1, $string.len()+1) as *mut u8 };

        unsafe { ::std::ptr::copy_nonoverlapping($string.as_ptr(), p, $string.len()); }

        $crate::cmdline::Token::Str(
            $crate::raw::Struct_cmdline_token_string {
                hdr: $crate::raw::Struct_cmdline_token_hdr {
                    ops: unsafe { &mut $crate::raw::cmdline_token_string_ops },
                    offset: offset_of!($container, $field) as u32,
                },
                string_data: $crate::raw::Struct_cmdline_token_string_data {
                    _str: p as *const i8,
                },
            }, ::std::marker::PhantomData
        )
    })
}

#[macro_export]
macro_rules! TOKEN_NUM_INITIALIZER {
    ($container:path, $field:ident, $numtype:expr) => (
        $crate::cmdline::Token::Num(
            $crate::raw::Struct_cmdline_token_num {
                hdr: $crate::raw::Struct_cmdline_token_hdr {
                    ops: unsafe { &mut $crate::raw::cmdline_token_string_ops },
                    offset: offset_of!($container, $field) as u32,
                },
                num_data: $crate::raw::Struct_cmdline_token_num_data {
                    _type: $numtype,
                },
            }, ::std::marker::PhantomData
        )
    )
}

#[macro_export]
macro_rules! TOKEN_IPADDR_INITIALIZER {
    ($container:path, $field:ident) => (
        TOKEN_IPADDR_INITIALIZER!($container,
                                  $field,
                                  $crate::raw::CMDLINE_IPADDR_V4 |
                                  $crate::raw::CMDLINE_IPADDR_V6)
    );

    ($container:path, $field:ident, $flags:expr) => (
        $crate::cmdline::Token::IpAddr(
            $crate::raw::Struct_cmdline_token_ipaddr {
                hdr: $crate::raw::Struct_cmdline_token_hdr {
                    ops: unsafe { &mut $crate::raw::cmdline_token_ipaddr_ops },
                    offset: offset_of!($container, $field) as u32,
                },
                ipaddr_data: $crate::raw::Struct_cmdline_token_ipaddr_data {
                    flags: $flags as u8,
                }
            }, ::std::marker::PhantomData
        )
    )
}

#[macro_export]
macro_rules! TOKEN_IPV4_INITIALIZER {
    ($container:path, $field:ident) => (
        TOKEN_IPADDR_INITIALIZER!($container,
                                  $field,
                                  $crate::raw::CMDLINE_IPADDR_V4)
    )
}

#[macro_export]
macro_rules! TOKEN_IPV6_INITIALIZER {
    ($container:path, $field:ident) => (
        TOKEN_IPADDR_INITIALIZER!($container,
                                  $field,
                                  $crate::raw::CMDLINE_IPADDR_V6)
    )
}

#[macro_export]
macro_rules! TOKEN_IPNET_INITIALIZER {
    ($container:path, $field:ident) => (
        TOKEN_IPADDR_INITIALIZER!($container,
                                  $field,
                                  $crate::raw::CMDLINE_IPADDR_V4 |
                                  $crate::raw::CMDLINE_IPADDR_V6 |
                                  $crate::raw::CMDLINE_IPADDR_NETWORK)
    )
}

#[macro_export]
macro_rules! TOKEN_IPV4NET_INITIALIZER {
    ($container:path, $field:ident) => (
        TOKEN_IPADDR_INITIALIZER!($container,
                                  $field,
                                  $crate::raw::CMDLINE_IPADDR_V4 |
                                  $crate::raw::CMDLINE_IPADDR_NETWORK)
    )
}

#[macro_export]
macro_rules! TOKEN_IPV6NET_INITIALIZER {
    ($container:path, $field:ident) => (
        TOKEN_IPADDR_INITIALIZER!($container,
                                  $field,
                                  $crate::raw::CMDLINE_IPADDR_V6 |
                                  $crate::raw::CMDLINE_IPADDR_NETWORK)
    )
}

#[macro_export]
macro_rules! TOKEN_ETHERADDR_INITIALIZER {
    ($container:path, $field:ident) => (
        $crate::cmdline::Token::EtherAddr(
            $crate::raw::Struct_cmdline_token_etheraddr {
                hdr: $crate::raw::Struct_cmdline_token_hdr {
                    ops: unsafe { &mut $crate::raw::cmdline_token_ipaddr_ops },
                    offset: offset_of!($container, $field) as u32,
                }
            }, ::std::marker::PhantomData
        )
    )
}

#[macro_export]
macro_rules! TOKEN_PORTLIST_INITIALIZER {
    ($container:path, $field:ident) => (
        $crate::cmdline::Token::PortList(
            $crate::raw::Struct_cmdline_token_portlist {
                hdr: $crate::raw::Struct_cmdline_token_hdr {
                    ops: unsafe { &mut $crate::raw::cmdline_token_portlist_ops },
                    offset: offset_of!($container, $field) as u32,
                }
            }, ::std::marker::PhantomData
        )
    )
}

pub type InstHandler<T, D> = fn(result: &mut T, cmdline: &RawCmdline, data: Option<*mut D>);

struct CommandHandlerContext<T, D> {
    data: Option<*mut D>,
    handler: InstHandler<T, D>,
}

extern "C" fn _command_handler_adapter<T, D>(result: &mut T,
                                             cl: *mut ffi::Struct_cmdline,
                                             ctxt: *mut CommandHandlerContext<T, D>) {
    unsafe {
        ((*ctxt).handler)(result, &RawCmdline(cl, false, false), (*ctxt).data);
    }
}

pub type RawInstPtr = *const ffi::Struct_cmdline_inst;

pub struct Inst(RawInstPtr);

impl Drop for Inst {
    fn drop(&mut self) {
        unsafe {
            libc::free((*self.0).help_str as *mut libc::c_void);
            libc::free(self.0 as *mut libc::c_void);
        }
    }
}

impl Inst {
    pub fn as_raw(&self) -> RawInstPtr {
        self.0
    }
}

pub fn inst<T, D>(handler: InstHandler<T, D>,
                  data: Option<*mut D>,
                  help: &'static str,
                  tokens: &[&Token<T>])
                  -> Inst {
    unsafe {
        let help_str = libc::calloc(1, help.len() + 1) as *mut c_char;

        ptr::copy_nonoverlapping(help.as_ptr(), help_str as *mut u8, help.len());

        let size = mem::size_of::<ffi::Struct_cmdline_inst>() +
                   mem::size_of::<RawTokenPtr>() * tokens.len();
        let inst = libc::calloc(1, size) as *mut ffi::Struct_cmdline_inst;

        *inst = ffi::Struct_cmdline_inst {
            f: mem::transmute(_command_handler_adapter::<T, D>),
            data: Box::into_raw(Box::new(CommandHandlerContext {
                data: data,
                handler: handler,
            })) as *mut c_void,
            help_str: help_str,
            tokens: ptr::null_mut(),
        };

        ptr::copy_nonoverlapping(tokens.iter()
                                       .map(|ref token| token.as_raw())
                                       .collect::<Vec<RawTokenPtr>>()
                                       .as_ptr(),
                                 mem::transmute(&((*inst).tokens)),
                                 tokens.len());

        Inst(inst)
    }
}

pub fn new(insts: &[&Inst]) -> Context {
    unsafe {
        let p = libc::calloc(insts.len() + 1, mem::size_of::<RawInstPtr>()) as *mut RawInstPtr;

        ptr::copy_nonoverlapping(insts.iter()
                                      .map(|ref inst| inst.as_raw())
                                      .collect::<Vec<RawInstPtr>>()
                                      .as_ptr(),
                                 p,
                                 insts.len());

        Context(p)
    }
}

pub struct Context(*const RawInstPtr);

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { libc::free(self.0 as *mut libc::c_void) }
    }
}

impl Context {
    pub fn open_stdin(&self, prompt: &str) -> RawCmdline {
        RawCmdline(unsafe {
                       ffi::cmdline_stdin_new(mem::transmute(self.0), prompt.as_ptr() as *const i8)
                   },
                   true,
                   true)
    }

    pub fn open_file<P: AsRef<Path>>(&self, prompt: &str, path: P) -> RawCmdline {
        RawCmdline(unsafe {
                       ffi::cmdline_file_new(mem::transmute(self.0),
                                         prompt.as_ptr() as *const i8,
                                         path.as_ref().as_os_str().to_str().unwrap().as_ptr() as *const i8)
                   },
                   true,
                   false)
    }
}

#[repr(i32)]
pub enum ReadlineStatus {
    Init = 0, // RDLINE_INIT
    Running = 1, // RDLINE_RUNNING
    Exited = 2, // RDLINE_EXITED
}

impl From<i32> for ReadlineStatus {
    fn from(status: i32) -> Self {
        unsafe { mem::transmute(status) }
    }
}

#[repr(i32)]
pub enum ParseStatus {
    Success = ffi::CMDLINE_PARSE_SUCCESS as i32,
    Ambiguous = ffi::CMDLINE_PARSE_AMBIGUOUS,
    NoMatch = ffi::CMDLINE_PARSE_NOMATCH,
    BadArgs = ffi::CMDLINE_PARSE_BAD_ARGS,
}

impl From<i32> for ParseStatus {
    fn from(status: i32) -> Self {
        unsafe { mem::transmute(status) }
    }
}

#[repr(i32)]
pub enum ParseCompleteState {
    TryToComplete = 0,
    DisplayChoice = -1,
}

impl From<i32> for ParseCompleteState {
    fn from(status: i32) -> Self {
        unsafe { mem::transmute(status) }
    }
}

#[repr(u32)]
pub enum ParseCompleteStatus {
    Finished = ffi::CMDLINE_PARSE_COMPLETE_FINISHED,
    Again = ffi::CMDLINE_PARSE_COMPLETE_AGAIN,
    Buffer = ffi::CMDLINE_PARSE_COMPLETED_BUFFER,
}

impl From<i32> for ParseCompleteStatus {
    fn from(status: i32) -> Self {
        unsafe { mem::transmute(status) }
    }
}

pub type RawCmdlinePtr = *mut ffi::Struct_cmdline;

pub struct RawCmdline(RawCmdlinePtr, bool, bool);

impl Drop for RawCmdline {
    fn drop(&mut self) {
        unsafe {
            if self.2 {
                ffi::cmdline_stdin_exit(self.0)
            }

            if self.1 {
                ffi::cmdline_free(self.0)
            }
        }
    }
}

extern "C" {
    fn _cmdline_write(cl: *const ffi::Struct_cmdline, s: *const c_char);
}

impl RawCmdline {
    pub fn print<T: string::ToString>(&self, s: T) -> Result<()> {
        unsafe {
            _cmdline_write(self.0, cstr!(s));
        }

        Ok(())
    }

    pub fn set_prompt(&self, s: &str) -> &RawCmdline {
        unsafe {
            ffi::cmdline_set_prompt(self.0, s.as_ptr() as *const i8);
        }

        self
    }

    pub fn interact(&self) -> &RawCmdline {
        unsafe {
            ffi::cmdline_interact(self.0);
        }

        self
    }

    pub fn poll(&self) -> Result<ReadlineStatus> {
        let status = unsafe { ffi::cmdline_poll(self.0) };

        rte_check!(status; ok => { ReadlineStatus::from(status) })
    }

    pub fn quit(&self) {
        unsafe { ffi::cmdline_quit(self.0) }
    }

    pub fn parse<T: string::ToString>(&self, buf: T) -> Result<()> {
        let status = unsafe { ffi::cmdline_parse(self.0, cstr!(buf)) };

        rte_check!(status; err => { Error::RteError(status) })
    }

    pub fn complete<T: string::ToString>(&self,
                                         buf: T,
                                         state: &mut ParseCompleteState,
                                         dst: &mut [u8])
                                         -> Result<ParseCompleteStatus> {
        let status = unsafe {
            ffi::cmdline_complete(self.0,
                                  cstr!(buf),
                                  mem::transmute(state),
                                  dst.as_mut_ptr() as *mut i8,
                                  dst.len() as u32)
        };

        rte_check!(status; ok => { ParseCompleteStatus::from(status) })
    }
}
