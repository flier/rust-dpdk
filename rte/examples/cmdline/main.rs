#[macro_use]
extern crate log;
extern crate env_logger;
extern crate libc;

#[macro_use]
extern crate rte;

use std::env;
use std::os::raw::c_void;

use rte::*;

struct CmdDelShowResult {
    action: cmdline::FixedStr,
}

fn cmd_obj_del_show(cl: &cmdline::RawCmdline, _: &CmdDelShowResult, _: Option<c_void>) {}

struct CmdObjAddResult {
    action: cmdline::FixedStr,
    name: cmdline::FixedStr,
    ip: cmdline::IpAddr,
}

fn cmd_obj_add(cl: &cmdline::RawCmdline, _: &CmdObjAddResult, _: Option<c_void>) {}

struct CmdHelpResult {
    help: cmdline::FixedStr,
}

fn cmd_help(cl: &cmdline::RawCmdline, _: &CmdHelpResult, _: Option<c_void>) {
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

fn cmd_quit(cl: &cmdline::RawCmdline, result: &CmdQuitResult, data: Option<c_void>) {
    cl.quit();
}

fn main() {
    env_logger::init().unwrap();

    let args: Vec<String> = env::args().collect();

    eal::init(&args).expect("Cannot init EAL");

    let cmds = &[&cmdline::inst(Some(cmd_obj_del_show),
                                None,
                                "Show/del an object",
                                &[&TOKEN_STRING_INITIALIZER!(CmdDelShowResult,
                                                             action,
                                                             "show#del")]),
                 &cmdline::inst(Some(cmd_obj_add),
                                None,
                                "Add an object (name, val)",
                                &[&TOKEN_STRING_INITIALIZER!(CmdObjAddResult, action, "add"),
                                  &TOKEN_STRING_INITIALIZER!(CmdObjAddResult, name, ""),
                                  &TOKEN_IPADDR_INITIALIZER!(CmdObjAddResult, ip)]),
                 &cmdline::inst(Some(cmd_help),
                                None,
                                "show help",
                                &[&TOKEN_STRING_INITIALIZER!(CmdHelpResult, help, "help")]),
                 &cmdline::inst(Some(cmd_quit),
                                None,
                                "quit",
                                &[&TOKEN_STRING_INITIALIZER!(CmdQuitResult, help, "quit")])];

    cmdline::new(cmds)
        .open_stdin("example> ")
        .interact();
}
