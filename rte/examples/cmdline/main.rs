#[macro_use]
extern crate log;
extern crate env_logger;
extern crate libc;

#[macro_use]
extern crate rte;

use std::env;
use std::net::IpAddr;
use std::os::raw::c_void;
use std::collections::HashMap;

use rte::*;

struct Object {
    name: String,
    ip: IpAddr,
}

type ObjectMap = HashMap<String, Object>;

struct CmdDelShowResult {
    action: cmdline::FixedStr,
}

fn cmd_obj_del_show_parsed(_: &cmdline::RawCmdline,
                           _: &mut CmdDelShowResult,
                           _: Option<*mut ObjectMap>) {
}

struct CmdObjAddResult {
    action: cmdline::FixedStr,
    name: cmdline::FixedStr,
    ip: cmdline::IpNetAddr,
}

fn cmd_obj_add_parsed(cl: &cmdline::RawCmdline,
                      res: &mut CmdObjAddResult,
                      data: Option<*mut ObjectMap>) {
    let objs = data.unwrap();

    let name = cmdline::str(&res.name).unwrap();

    unsafe {

        if (*objs).contains_key(name) {
            cl.print(format!("Object {} already exist\n", name)).unwrap();

            return;
        }

        let ip = cmdline::ipaddr(&mut res.ip);

        let _ = (*objs).insert(String::from(name),
                               Object {
                                   name: String::from(name),
                                   ip: ip,
                               });

        cl.print(format!("Object {} added, ip={}\n", name, ip)).unwrap();
    }
}

struct CmdHelpResult {
    help: cmdline::FixedStr,
}

fn cmd_help_parsed(cl: &cmdline::RawCmdline, _: &mut CmdHelpResult, _: Option<*mut c_void>) {
    cl.print(r#"Demo example of command line interface in RTE


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
"#)
      .unwrap();
}

struct CmdQuitResult {
    help: cmdline::FixedStr,
}

fn cmd_quit_parsed(cl: &cmdline::RawCmdline, _: &mut CmdQuitResult, _: Option<*mut c_void>) {
    cl.quit();
}

fn main() {
    env_logger::init().unwrap();

    let args: Vec<String> = env::args().collect();

    eal::init(&args).expect("Cannot init EAL");

    let mut objects = ObjectMap::new();

    let cmd_obj_action = TOKEN_STRING_INITIALIZER!(CmdDelShowResult, action, "show#del");

    let cmd_obj_del_show = cmdline::inst(cmd_obj_del_show_parsed,
                                         Some(&mut objects),
                                         "Show/del an object",
                                         &[&cmd_obj_action]);

    let cmd_obj_action_add = TOKEN_STRING_INITIALIZER!(CmdObjAddResult, action, "add");
    let cmd_obj_name = TOKEN_STRING_INITIALIZER!(CmdObjAddResult, name);
    let cmd_obj_ip = TOKEN_IPADDR_INITIALIZER!(CmdObjAddResult, ip);

    let cmd_obj_add = cmdline::inst(cmd_obj_add_parsed,
                                    Some(&mut objects),
                                    "Add an object (name, val)",
                                    &[&cmd_obj_action_add, &cmd_obj_name, &cmd_obj_ip]);

    let cmd_help_help = TOKEN_STRING_INITIALIZER!(CmdHelpResult, help, "help");

    let cmd_help = cmdline::inst(cmd_help_parsed, None, "show help", &[&cmd_help_help]);

    let cmd_quit_quit = TOKEN_STRING_INITIALIZER!(CmdQuitResult, help, "quit");

    let cmd_quit = cmdline::inst(cmd_quit_parsed, None, "quit", &[&cmd_quit_quit]);

    let cmds = &[&cmd_obj_del_show, &cmd_obj_add, &cmd_help, &cmd_quit];

    cmdline::new(cmds)
        .open_stdin("example> ")
        .interact();
}
