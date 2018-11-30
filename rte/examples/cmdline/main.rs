extern crate libc;
extern crate pretty_env_logger;
extern crate rte;

use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::ffi::CString;
use std::marker::PhantomData;
use std::mem;
use std::net::IpAddr;
use std::os::raw::c_void;
use std::rc::Rc;
use std::slice;
use std::str;

use rte::cmdline::*;
use rte::*;

struct Object {
    name: String,
    ip: IpAddr,
}

type ObjectMap = HashMap<String, Object>;

struct TokenObjectListData {
    objs: Rc<RefCell<ObjectMap>>,
}

struct TokenObjectList {
    hdr: cmdline::RawTokenHeader,
    obj_list_data: TokenObjectListData,
}

unsafe extern "C" fn parse_obj_list(
    token: *mut RawParseTokenHeader,
    srcbuf: *const i8,
    res: *mut libc::c_void,
    ressize: u32,
) -> i32 {
    if srcbuf.is_null() {
        return -1;
    }

    if !res.is_null() && (ressize as usize) < mem::size_of::<*const Object>() {
        return -1;
    }

    let mut p = srcbuf as *const u8;
    let mut token_len = 0;

    while !cmdline::is_end_of_token(*p) {
        p = p.offset(1);
        token_len += 1;
    }

    let name = str::from_utf8(slice::from_raw_parts(srcbuf as *const u8, token_len)).unwrap();

    if let Some(obj) = (token as *mut TokenObjectList)
        .as_ref()
        .unwrap()
        .obj_list_data
        .objs
        .borrow()
        .get(name)
    {
        if !res.is_null() {
            (res as *mut *const Object).write(obj);
        }

        token_len as i32
    } else {
        -1
    }
}

unsafe extern "C" fn complete_get_nb_obj_list(token: *mut RawTokenHeader) -> i32 {
    (token as *mut TokenObjectList)
        .as_ref()
        .unwrap()
        .obj_list_data
        .objs
        .borrow()
        .len() as i32
}

unsafe extern "C" fn complete_get_elt_obj_list(
    token: *mut cmdline::RawTokenHeader,
    idx: i32,
    dstbuf: *mut i8,
    size: u32,
) -> i32 {
    if let Some((name, _)) = (token as *mut TokenObjectList)
        .as_ref()
        .unwrap()
        .obj_list_data
        .objs
        .borrow()
        .iter()
        .nth(idx as usize)
    {
        if (name.len() + 1) < size as usize {
            let buf = slice::from_raw_parts_mut(dstbuf as *mut u8, size as usize);

            buf[..name.len()].clone_from_slice(name.as_bytes());
            buf[name.len()] = 0;

            return 0;
        }
    }

    -1
}

unsafe extern "C" fn get_help_obj_list(
    _: *mut cmdline::RawTokenHeader,
    dstbuf: *mut i8,
    size: u32,
) -> i32 {
    let dbuf = slice::from_raw_parts_mut(dstbuf as *mut u8, size as usize);
    let s = CString::new("Obj-List").unwrap();
    let sbuf = s.as_bytes_with_nul();

    if sbuf.len() < size as usize {
        dbuf[0..sbuf.len()].clone_from_slice(sbuf);

        0
    } else {
        -1
    }
}

struct CmdDelShowResult<'a> {
    action: cmdline::FixedStr,
    obj: &'a Object,
}

impl<'a> CmdDelShowResult<'a> {
    fn parsed(&mut self, cl: &cmdline::CmdLine, objs: Option<&RefCell<ObjectMap>>) {
        let action = self.action.to_str();

        match action {
            "show" => {
                cl.print(format!("Object {}, ip={}\n", self.obj.name, self.obj.ip))
                    .unwrap();
            }
            "del" => {
                if let Some(ref obj) = objs.unwrap().borrow_mut().remove(&self.obj.name) {
                    cl.print(format!("Object {} removed, ip={}\n", obj.name, obj.ip))
                        .unwrap();
                }
            }
            _ => {
                cl.print(format!("Unknown action, {}", action)).unwrap();
            }
        }
    }
}

struct CmdObjAddResult {
    action: cmdline::FixedStr,
    name: cmdline::FixedStr,
    ip: cmdline::IpNetAddr,
}

impl CmdObjAddResult {
    fn parsed(&mut self, cl: &cmdline::CmdLine, objs: Option<&RefCell<ObjectMap>>) {
        let name = self.name.to_str();

        if objs.unwrap().borrow().contains_key(name) {
            cl.print(format!("Object {} already exist\n", name))
                .unwrap();

            return;
        }

        let obj = Object {
            name: String::from(name),
            ip: self.ip.to_ipaddr(),
        };

        cl.print(format!("Object {} added, ip={}\n", name, obj.ip))
            .unwrap();

        let _ = objs.unwrap().borrow_mut().insert(String::from(name), obj);
    }
}

struct CmdHelpResult {
    help: cmdline::FixedStr,
}

impl CmdHelpResult {
    fn parsed(&mut self, cl: &cmdline::CmdLine, _: Option<&c_void>) {
        cl.print(
            r#"Demo example of command line interface in RTE


This is a readline-like interface that can be used to
debug your RTE application. It supports some features
of GNU readline like completion, cut/paste, and some
other special bindings.

This demo shows how rte_cmdline library can be
extended to handle a list of objects. There are
3 commands:
- add obj_name IP
- del obj_name
- show obj_name
"#,
        ).unwrap();
    }
}

struct CmdQuitResult {
    quit: cmdline::FixedStr,
}

impl CmdQuitResult {
    fn parsed(&mut self, cl: &cmdline::CmdLine, _: Option<&c_void>) {
        cl.quit();
    }
}

fn main() {
    pretty_env_logger::init();

    let args: Vec<String> = env::args().collect();

    eal::init(&args).expect("Cannot init EAL");

    let objects = Rc::new(RefCell::new(ObjectMap::new()));

    let cmd_obj_action = TOKEN_STRING_INITIALIZER!(CmdDelShowResult, action, "show#del");

    let mut token_obj_list_ops = cmdline::RawTokenOps {
        parse: Some(parse_obj_list),
        complete_get_nb: Some(complete_get_nb_obj_list),
        complete_get_elt: Some(complete_get_elt_obj_list),
        get_help: Some(get_help_obj_list),
    };

    let token_obj_list = TokenObjectList {
        hdr: cmdline::RawTokenHeader {
            ops: &mut token_obj_list_ops,
            offset: offset_of!(CmdDelShowResult, obj) as u32,
        },
        obj_list_data: TokenObjectListData {
            objs: objects.clone(),
        },
    };

    let cmd_obj_obj = cmdline::Token::Raw(&token_obj_list.hdr, PhantomData);

    let cmd_obj_del_show = cmdline::inst(
        CmdDelShowResult::parsed,
        Some(&objects),
        "Show/del an object",
        &[&cmd_obj_action, &cmd_obj_obj],
    );

    let cmd_obj_action_add = TOKEN_STRING_INITIALIZER!(CmdObjAddResult, action, "add");
    let cmd_obj_name = TOKEN_STRING_INITIALIZER!(CmdObjAddResult, name);
    let cmd_obj_ip = TOKEN_IPADDR_INITIALIZER!(CmdObjAddResult, ip);

    let cmd_obj_add = cmdline::inst(
        CmdObjAddResult::parsed,
        Some(&objects),
        "Add an object (name, val)",
        &[&cmd_obj_action_add, &cmd_obj_name, &cmd_obj_ip],
    );

    let cmd_help_help = TOKEN_STRING_INITIALIZER!(CmdHelpResult, help, "help");

    let cmd_help = cmdline::inst(CmdHelpResult::parsed, None, "show help", &[&cmd_help_help]);

    let cmd_quit_quit = TOKEN_STRING_INITIALIZER!(CmdQuitResult, quit, "quit");

    let cmd_quit = cmdline::inst(CmdQuitResult::parsed, None, "quit", &[&cmd_quit_quit]);

    let cmds = &[&cmd_obj_del_show, &cmd_obj_add, &cmd_help, &cmd_quit];

    cmdline::new(cmds)
        .open_stdin("example> ")
        .expect("fail to open stdin")
        .interact();
}
