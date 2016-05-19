use std::mem;
use std::ptr;
use std::path::Path;

use ffi;

pub type RawTokenPtr = *const ffi::Struct_cmdline_token_hdr;
pub type RawStrToken = ffi::Struct_cmdline_token_string;
pub type RawNumToken = ffi::Struct_cmdline_token_num;
pub type RawIpAddrToken = ffi::Struct_cmdline_token_ipaddr;
pub type RawEtherAddrToken = ffi::Struct_cmdline_token_etheraddr;

pub enum Token {
    Str(RawStrToken),
    Num(RawNumToken),
    IpAddr(RawIpAddrToken),
    EtherAddr(RawEtherAddrToken),
}

impl Token {
    pub fn as_raw(&self) -> RawTokenPtr {
        match self {
            &Token::Str(ref token) => &token.hdr,
            &Token::Num(ref token) => &token.hdr,
            &Token::IpAddr(ref token) => &token.hdr,
            &Token::EtherAddr(ref token) => &token.hdr,
        }
    }
}

pub type FixedStr = ffi::cmdline_fixed_string_t;
pub type IpAddr = ffi::cmdline_ipaddr_t;

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
    ($container:path, $field:ident, $string:expr) => (
        $crate::cmdline::Token::Str(
            $crate::raw::Struct_cmdline_token_string {
                hdr: $crate::raw::Struct_cmdline_token_hdr {
                    ops: unsafe { &mut $crate::raw::cmdline_token_string_ops },
                    offset: offset_of!($container, $field) as u32,
                },
                string_data: $crate::raw::Struct_cmdline_token_string_data {
                    _str: $string.as_ptr() as *const i8,
                },
            }
        )
    )
}

#[macro_export]
macro_rules! TOKEN_IPADDR_INITIALIZER {
    ($container:path, $field:ident) => (
        TOKEN_IPADDR_INITIALIZER!($container,
                                  $field,
                                  ($crate::raw::CMDLINE_IPADDR_V4 | $crate::raw::CMDLINE_IPADDR_V6) as u8)
    );

    ($container:path, $field:ident, $flags:expr) => (
        $crate::cmdline::Token::IpAddr(
            $crate::raw::Struct_cmdline_token_ipaddr {
                hdr: $crate::raw::Struct_cmdline_token_hdr {
                    ops: unsafe { &mut $crate::raw::cmdline_token_ipaddr_ops },
                    offset: offset_of!($container, $field) as u32,
                },
                ipaddr_data: $crate::raw::Struct_cmdline_token_ipaddr_data {
                    flags: $flags,
                }
            }
        )
    )
}

#[macro_export]
macro_rules! TOKEN_IPV4_INITIALIZER {
    ($container:path, $field:ident) => (
        TOKEN_IPADDR_INITIALIZER!($container,
                                  $field,
                                  $crate::raw::CMDLINE_IPADDR_V4 as u8)
    )
}

#[macro_export]
macro_rules! TOKEN_IPV6_INITIALIZER {
    ($container:path, $field:ident) => (
        TOKEN_IPADDR_INITIALIZER!($container,
                                  $field,
                                  $crate::raw::CMDLINE_IPADDR_V6 as u8)
    )
}

#[macro_export]
macro_rules! TOKEN_IPNET_INITIALIZER {
    ($container:path, $field:ident) => (
        TOKEN_IPADDR_INITIALIZER!($container,
                                  $field,
                                  ($crate::raw::CMDLINE_IPADDR_V4 |
                                   $crate::raw::CMDLINE_IPADDR_V6 |
                                   $crate::raw::CMDLINE_IPADDR_NETWORK) as u8)
    )
}

#[macro_export]
macro_rules! TOKEN_IPV4NET_INITIALIZER {
    ($container:path, $field:ident) => (
        TOKEN_IPADDR_INITIALIZER!($container,
                                  $field,
                                  ($crate::raw::CMDLINE_IPADDR_V4 |
                                   $crate::raw::CMDLINE_IPADDR_NETWORK) as u8)
    )
}

#[macro_export]
macro_rules! TOKEN_IPV6NET_INITIALIZER {
    ($container:path, $field:ident) => (
        TOKEN_IPADDR_INITIALIZER!($container,
                                  $field,
                                  ($crate::raw::CMDLINE_IPADDR_V6 |
                                   $crate::raw::CMDLINE_IPADDR_NETWORK) as u8)
    )
}

pub type InstHandler<T, D> = extern "C" fn(result: &T, cmdline: &RawCmdline, data: *const D);

pub type RawInst = ffi::Struct_cmdline_inst;
pub type RawInstPtr = *const ffi::Struct_cmdline_inst;

pub struct Inst(RawInst, Box<Vec<RawTokenPtr>>);

impl Inst {
    pub fn as_raw(&self) -> RawInstPtr {
        &self.0
    }
}

impl Inst {
    pub fn new<T, D>(handler: Option<InstHandler<T, D>>,
                     data: Option<&D>,
                     help: &'static str,
                     tokens: &[&Token])
                     -> Inst {
        unsafe {
            let mut tokens: Box<Vec<RawTokenPtr>> = Box::new(tokens.iter()
                                                                   .map(|ref token| {
                                                                       token.as_raw()
                                                                   })
                                                                   .collect());

            tokens.push(ptr::null());

            let mut inst = Inst(ffi::Struct_cmdline_inst {
                                    f: mem::transmute(handler),
                                    data: mem::transmute(data),
                                    help_str: help.as_ptr() as *const i8,
                                    tokens: ptr::null_mut(),
                                },
                                tokens);

            inst.0.tokens = mem::transmute(inst.1.as_ptr());

            inst
        }
    }
}

pub struct Context(Box<Vec<RawInstPtr>>);

pub fn new(insts: &[&Inst]) -> Context {
    let mut insts: Box<Vec<RawInstPtr>> = Box::new(insts.iter()
                                                        .map(|ref inst| inst.as_raw())
                                                        .collect());

    insts.push(ptr::null());

    Context(insts)
}

impl Context {
    pub fn open_stdin(&self, prompt: &str) -> RawCmdline {
        RawCmdline(unsafe {
                       ffi::cmdline_stdin_new(mem::transmute(self.0.as_ptr()),
                                              prompt.as_ptr() as *const i8)
                   },
                   true)
    }

    pub fn open_file<P: AsRef<Path>>(&self, prompt: &str, path: P) -> RawCmdline {
        RawCmdline(unsafe {
                       ffi::cmdline_file_new(mem::transmute(self.0.as_ptr()),
                                         prompt.as_ptr() as *const i8,
                                         path.as_ref().as_os_str().to_str().unwrap().as_ptr() as *const i8)
                   },
                   false)
    }
}

pub type RawCmdlinePtr = *mut ffi::Struct_cmdline;

pub struct RawCmdline(RawCmdlinePtr, bool);

impl Drop for RawCmdline {
    fn drop(&mut self) {
        unsafe {
            if self.1 {
                ffi::cmdline_stdin_exit(self.0)
            }

            ffi::cmdline_free(self.0)
        }
    }
}

impl RawCmdline {
    pub fn print(&self, s: &str) {
        unsafe { ffi::cmdline_printf(self.0, s.as_ptr() as *const i8) }
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

    pub fn quit(&self) {
        unsafe { ffi::cmdline_quit(self.0) }
    }
}
