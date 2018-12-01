use std::ffi::CStr;
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ops::{Deref, DerefMut};
use std::os::raw::{c_char, c_void};
use std::path::Path;
use std::ptr;
use std::string;

use libc;

use ffi;

use errors::{AsResult, ErrorKind::CmdLineParseError, Result};
use ether;

pub type RawTokenHeader = ffi::cmdline_token_hdr;
pub type RawTokenPtr = *const RawTokenHeader;
pub type RawStrToken = ffi::cmdline_token_string;
pub type RawNumToken = ffi::cmdline_token_num;
pub type RawIpAddrToken = ffi::cmdline_token_ipaddr;
pub type RawEtherAddrToken = ffi::cmdline_token_etheraddr;
pub type RawPortListToken = ffi::cmdline_token_portlist;
pub type RawParseTokenHeader = ffi::cmdline_parse_token_hdr_t;

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
            unsafe { libc::free(token.string_data.str as *mut libc::c_void) }
        }
    }
}

pub type NumType = ffi::cmdline_numtype::Type;

pub type RawFixedStr = ffi::cmdline_fixed_string_t;
pub type RawIpNetAddr = ffi::cmdline_ipaddr_t;
pub type RawEtherAddr = ffi::ether_addr;
pub type RawPortList = ffi::cmdline_portlist_t;

pub struct FixedStr(RawFixedStr);

impl Deref for FixedStr {
    type Target = RawFixedStr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for FixedStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl FixedStr {
    pub fn to_str(&self) -> &str {
        unsafe { CStr::from_ptr(self.0.as_ptr()).to_str().unwrap() }
    }
}

pub struct IpNetAddr(RawIpNetAddr);

impl Deref for IpNetAddr {
    type Target = RawIpNetAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for IpNetAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_ipaddr())
    }
}

impl IpNetAddr {
    pub fn as_ipv4(&self) -> &Ipv4Addr {
        unsafe { mem::transmute(&self.0.addr) }
    }

    pub fn as_ipv6(&self) -> &Ipv6Addr {
        unsafe { mem::transmute(&self.0.addr) }
    }

    pub fn to_ipaddr(&self) -> IpAddr {
        if self.0.family == libc::AF_INET as u8 {
            IpAddr::V4(*self.as_ipv4())
        } else {
            IpAddr::V6(*self.as_ipv6())
        }
    }
}

pub struct EtherAddr(RawEtherAddr);

impl Deref for EtherAddr {
    type Target = ether::EtherAddr;

    fn deref(&self) -> &Self::Target {
        unsafe { mem::transmute(&self.0) }
    }
}

impl fmt::Display for EtherAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_etheraddr())
    }
}

impl EtherAddr {
    pub fn to_etheraddr(&self) -> ether::EtherAddr {
        ether::EtherAddr::from(self.0.addr_bytes)
    }
}

pub struct PortList(RawPortList);

impl PortList {
    pub fn to_portlist<'a>(&'a self) -> Box<Iterator<Item = u32> + 'a> {
        Box::new((0..32).filter(move |portid| ((1 << portid) as u32 & self.0.map) != 0))
    }
}

impl fmt::Display for PortList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.to_portlist()
                .map(|portid| portid.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

pub fn is_end_of_token(c: u8) -> bool {
    unsafe { ffi::cmdline_isendoftoken(c as i8) != 0 }
}

pub type RawTokenOps = ffi::cmdline_token_ops;

#[macro_export]
macro_rules! TOKEN_STRING_INITIALIZER {
    ($container:path, $field:ident) => {{
        $crate::cmdline::Token::Str(
            $crate::raw::cmdline_token_string {
                hdr: $crate::raw::cmdline_token_hdr {
                    ops: unsafe { &mut $crate::raw::cmdline_token_string_ops },
                    offset: offset_of!($container, $field) as u32,
                },
                string_data: $crate::raw::cmdline_token_string_data {
                    str: ::std::ptr::null(),
                },
            },
            ::std::marker::PhantomData,
        )
    }};

    ($container:path, $field:ident, $string:expr) => {{
        let p = unsafe { ::libc::calloc(1, $string.len() + 1) as *mut u8 };

        unsafe {
            ::std::ptr::copy_nonoverlapping($string.as_ptr(), p, $string.len());
        }

        $crate::cmdline::Token::Str(
            $crate::raw::cmdline_token_string {
                hdr: $crate::raw::cmdline_token_hdr {
                    ops: unsafe { &mut $crate::raw::cmdline_token_string_ops },
                    offset: offset_of!($container, $field) as u32,
                },
                string_data: $crate::raw::cmdline_token_string_data {
                    str: p as *const i8,
                },
            },
            ::std::marker::PhantomData,
        )
    }};
}

#[macro_export]
macro_rules! TOKEN_NUM_INITIALIZER {
    ($container:path, $field:ident, u8) => {
        TOKEN_NUM_INITIALIZER!($container, $field, $crate::raw::cmdline_numtype::UINT8)
    };
    ($container:path, $field:ident, u16) => {
        TOKEN_NUM_INITIALIZER!($container, $field, $crate::raw::cmdline_numtype::UINT16)
    };
    ($container:path, $field:ident, u32) => {
        TOKEN_NUM_INITIALIZER!($container, $field, $crate::raw::cmdline_numtype::UINT32)
    };
    ($container:path, $field:ident, u64) => {
        TOKEN_NUM_INITIALIZER!($container, $field, $crate::raw::cmdline_numtype::UINT64)
    };
    ($container:path, $field:ident, i8) => {
        TOKEN_NUM_INITIALIZER!($container, $field, $crate::raw::cmdline_numtype::INT8)
    };
    ($container:path, $field:ident, i16) => {
        TOKEN_NUM_INITIALIZER!($container, $field, $crate::raw::cmdline_numtype::INT16)
    };
    ($container:path, $field:ident, i32) => {
        TOKEN_NUM_INITIALIZER!($container, $field, $crate::raw::cmdline_numtype::INT32)
    };
    ($container:path, $field:ident, i64) => {
        TOKEN_NUM_INITIALIZER!($container, $field, $crate::raw::cmdline_numtype::INT64)
    };

    ($container:path, $field:ident, $numtype:expr) => {
        $crate::cmdline::Token::Num(
            $crate::raw::cmdline_token_num {
                hdr: $crate::raw::cmdline_token_hdr {
                    ops: unsafe { &mut $crate::raw::cmdline_token_num_ops },
                    offset: offset_of!($container, $field) as u32,
                },
                num_data: $crate::raw::cmdline_token_num_data { type_: $numtype },
            },
            ::std::marker::PhantomData,
        )
    };
}

#[macro_export]
macro_rules! TOKEN_IPADDR_INITIALIZER {
    ($container:path, $field:ident) => {
        TOKEN_IPADDR_INITIALIZER!(
            $container,
            $field,
            $crate::raw::CMDLINE_IPADDR_V4 | $crate::raw::CMDLINE_IPADDR_V6
        )
    };

    ($container:path, $field:ident, $flags:expr) => {
        $crate::cmdline::Token::IpAddr(
            $crate::raw::cmdline_token_ipaddr {
                hdr: $crate::raw::cmdline_token_hdr {
                    ops: unsafe { &mut $crate::raw::cmdline_token_ipaddr_ops },
                    offset: offset_of!($container, $field) as u32,
                },
                ipaddr_data: $crate::raw::cmdline_token_ipaddr_data {
                    flags: $flags as u8,
                },
            },
            ::std::marker::PhantomData,
        )
    };
}

#[macro_export]
macro_rules! TOKEN_IPV4_INITIALIZER {
    ($container:path, $field:ident) => {
        TOKEN_IPADDR_INITIALIZER!($container, $field, $crate::raw::CMDLINE_IPADDR_V4)
    };
}

#[macro_export]
macro_rules! TOKEN_IPV6_INITIALIZER {
    ($container:path, $field:ident) => {
        TOKEN_IPADDR_INITIALIZER!($container, $field, $crate::raw::CMDLINE_IPADDR_V6)
    };
}

#[macro_export]
macro_rules! TOKEN_IPNET_INITIALIZER {
    ($container:path, $field:ident) => {
        TOKEN_IPADDR_INITIALIZER!(
            $container,
            $field,
            $crate::raw::CMDLINE_IPADDR_V4
                | $crate::raw::CMDLINE_IPADDR_V6
                | $crate::raw::CMDLINE_IPADDR_NETWORK
        )
    };
}

#[macro_export]
macro_rules! TOKEN_IPV4NET_INITIALIZER {
    ($container:path, $field:ident) => {
        TOKEN_IPADDR_INITIALIZER!(
            $container,
            $field,
            $crate::raw::CMDLINE_IPADDR_V4 | $crate::raw::CMDLINE_IPADDR_NETWORK
        )
    };
}

#[macro_export]
macro_rules! TOKEN_IPV6NET_INITIALIZER {
    ($container:path, $field:ident) => {
        TOKEN_IPADDR_INITIALIZER!(
            $container,
            $field,
            $crate::raw::CMDLINE_IPADDR_V6 | $crate::raw::CMDLINE_IPADDR_NETWORK
        )
    };
}

#[macro_export]
macro_rules! TOKEN_ETHERADDR_INITIALIZER {
    ($container:path, $field:ident) => {
        $crate::cmdline::Token::EtherAddr(
            $crate::raw::cmdline_token_etheraddr {
                hdr: $crate::raw::cmdline_token_hdr {
                    ops: unsafe { &mut $crate::raw::cmdline_token_etheraddr_ops },
                    offset: offset_of!($container, $field) as u32,
                },
            },
            ::std::marker::PhantomData,
        )
    };
}

#[macro_export]
macro_rules! TOKEN_PORTLIST_INITIALIZER {
    ($container:path, $field:ident) => {
        $crate::cmdline::Token::PortList(
            $crate::raw::cmdline_token_portlist {
                hdr: $crate::raw::cmdline_token_hdr {
                    ops: unsafe { &mut $crate::raw::cmdline_token_portlist_ops },
                    offset: offset_of!($container, $field) as u32,
                },
            },
            ::std::marker::PhantomData,
        )
    };
}

pub type InstHandler<T, D> = fn(inst: &mut T, cmdline: &CmdLine, data: Option<&D>);

struct InstHandlerContext<'a, T, D>
where
    D: 'a,
{
    handler: InstHandler<T, D>,
    data: Option<&'a D>,
}

unsafe extern "C" fn _inst_handler_stub<T, D>(
    inst: *mut c_void,
    cl: *mut RawCmdLine,
    ctxt: *mut c_void,
) {
    let ctxt = Box::from_raw(ctxt as *mut InstHandlerContext<T, D>);

    (ctxt.handler)(
        (inst as *mut T).as_mut().unwrap(),
        &CmdLine::Borrowed(cl),
        ctxt.data,
    );
}

pub type RawInstPtr = *const ffi::cmdline_inst;

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

pub fn inst<T, D>(
    handler: InstHandler<T, D>,
    data: Option<&D>,
    help: &'static str,
    tokens: &[&Token<T>],
) -> Inst {
    unsafe {
        let help_str = libc::calloc(1, help.len() + 1) as *mut c_char;

        ptr::copy_nonoverlapping(help.as_ptr(), help_str as *mut u8, help.len());

        let size =
            mem::size_of::<ffi::cmdline_inst>() + mem::size_of::<RawTokenPtr>() * tokens.len();
        let inst = libc::calloc(1, size) as *mut ffi::cmdline_inst;

        *inst = ffi::cmdline_inst {
            f: Some(_inst_handler_stub::<T, D>),
            data: Box::into_raw(Box::new(InstHandlerContext { data, handler })) as *mut _,
            help_str: help_str,
            tokens: ffi::__IncompleteArrayField::new(),
        };

        ptr::copy_nonoverlapping(
            tokens
                .iter()
                .map(|ref token| token.as_raw())
                .collect::<Vec<RawTokenPtr>>()
                .as_ptr(),
            mem::transmute(&((*inst).tokens)),
            tokens.len(),
        );

        Inst(inst)
    }
}

pub fn new(insts: &[&Inst]) -> Context {
    unsafe {
        let p = libc::calloc(insts.len() + 1, mem::size_of::<RawInstPtr>()) as *mut RawInstPtr;

        ptr::copy_nonoverlapping(
            insts
                .iter()
                .map(|ref inst| inst.as_raw())
                .collect::<Vec<RawInstPtr>>()
                .as_ptr(),
            p,
            insts.len(),
        );

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
    pub fn open_stdin(&self, prompt: &str) -> Result<StdInCmdLine> {
        let cl = unsafe { ffi::cmdline_stdin_new(mem::transmute(self.0), try!(to_cptr!(prompt))) };

        rte_check!(cl, NonNull; ok => { StdInCmdLine(CmdLine::Owned(cl)) })
    }

    pub fn open_file<P: AsRef<Path>>(&self, prompt: &str, path: P) -> Result<CmdLine> {
        let cl = unsafe {
            ffi::cmdline_file_new(
                mem::transmute(self.0),
                try!(to_cptr!(prompt)),
                path.as_ref().as_os_str().to_str().unwrap().as_ptr() as *const i8,
            )
        };

        rte_check!(cl, NonNull; ok => { CmdLine::Owned(cl) })
    }
}

pub struct StdInCmdLine(CmdLine);

impl Drop for StdInCmdLine {
    fn drop(&mut self) {
        unsafe { ffi::cmdline_stdin_exit(self.0.as_raw()) }
    }
}

impl Deref for StdInCmdLine {
    type Target = CmdLine;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StdInCmdLine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[repr(i32)]
pub enum ReadlineStatus {
    Init = 0,    // RDLINE_INIT
    Running = 1, // RDLINE_RUNNING
    Exited = 2,  // RDLINE_EXITED
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

pub type RawCmdLine = ffi::cmdline;
pub type RawCmdLinePtr = *mut ffi::cmdline;

pub enum CmdLine {
    Owned(RawCmdLinePtr),
    Borrowed(RawCmdLinePtr),
}

impl Drop for CmdLine {
    fn drop(&mut self) {
        if let &mut CmdLine::Owned(cl) = self {
            unsafe { ffi::cmdline_free(cl) }
        }
    }
}

impl Deref for CmdLine {
    type Target = RawCmdLine;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.as_raw() }
    }
}

impl DerefMut for CmdLine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.as_raw() }
    }
}

impl CmdLine {
    pub fn as_raw(&self) -> RawCmdLinePtr {
        match self {
            &CmdLine::Owned(cl) | &CmdLine::Borrowed(cl) => cl,
        }
    }

    pub fn print<T: string::ToString>(&self, s: T) -> Result<&Self> {
        unsafe {
            ffi::cmdline_printf(self.as_raw(), try!(to_cptr!(s.to_string())));
        }

        Ok(self)
    }

    pub fn println<T: string::ToString>(&self, s: T) -> Result<&Self> {
        unsafe {
            ffi::cmdline_printf(
                self.as_raw(),
                try!(to_cptr!(format!("{}\n", s.to_string()))),
            );
        }

        Ok(self)
    }

    pub fn set_prompt(&self, s: &str) -> &CmdLine {
        unsafe {
            ffi::cmdline_set_prompt(self.as_raw(), s.as_ptr() as *const i8);
        }

        self
    }

    pub fn interact(&self) -> &CmdLine {
        unsafe {
            ffi::cmdline_interact(self.as_raw());
        }

        self
    }

    pub fn poll(&self) -> Result<ReadlineStatus> {
        let status = unsafe { ffi::cmdline_poll(self.as_raw()) };

        rte_check!(status; ok => { ReadlineStatus::from(status) })
    }

    pub fn quit(&self) {
        unsafe { ffi::cmdline_quit(self.as_raw()) }
    }

    pub fn parse<T: string::ToString>(&self, buf: T) -> Result<&Self> {
        let status = unsafe { ffi::cmdline_parse(self.as_raw(), try!(to_cptr!(buf.to_string()))) };

        status.ok_or(CmdLineParseError(status)).map(|_| self)
    }

    pub fn complete<T: string::ToString>(
        &self,
        buf: T,
        state: &mut ParseCompleteState,
        dst: &mut [u8],
    ) -> Result<ParseCompleteStatus> {
        let status = unsafe {
            ffi::cmdline_complete(
                self.as_raw(),
                try!(to_cptr!(buf.to_string())),
                mem::transmute(state),
                dst.as_mut_ptr() as *mut i8,
                dst.len() as u32,
            )
        };

        rte_check!(status; ok => { ParseCompleteStatus::from(status) })
    }
}
