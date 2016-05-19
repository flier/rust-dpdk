#[macro_use]
extern crate rte;

use std::env;
use std::os::raw::c_void;

use rte::*;

struct CmdDelShowResult {
    action: cmdline::FixedStr,
}

extern "C" fn cmd_obj_del_show_parsed(result: &CmdDelShowResult,
                                      cl: &cmdline::RawCmdline,
                                      data: *const c_void) {
}

struct CmdObjAddResult {
    action: cmdline::FixedStr,
    name: cmdline::FixedStr,
    ip: cmdline::IpAddr,
}

extern "C" fn cmd_obj_add_parsed(result: &CmdObjAddResult,
                                 cl: &cmdline::RawCmdline,
                                 data: *const c_void) {
}

struct CmdHelpResult {
    help: cmdline::FixedStr,
}

extern "C" fn cmd_help_parsed(result: &CmdHelpResult,
                              cl: &cmdline::RawCmdline,
                              data: *const c_void) {
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
"#);
}

fn handle_commands() {
    let cmd_obj_action = TOKEN_STRING_INITIALIZER!(CmdDelShowResult, action, "show#del");

    let cmd_obj_del_show = cmdline::Inst::new(Some(cmd_obj_del_show_parsed),
                                              None,
                                              "Show/del an object",
                                              &[&cmd_obj_action]);

    let cmd_obj_action_add = TOKEN_STRING_INITIALIZER!(CmdObjAddResult, action, "add");
    let cmd_obj_name = TOKEN_STRING_INITIALIZER!(CmdObjAddResult, name, "");
    let cmd_obj_ip = TOKEN_IPADDR_INITIALIZER!(CmdObjAddResult, ip);

    let cmd_obj_add = cmdline::Inst::new(Some(cmd_obj_add_parsed),
                                         None,
                                         "Add an object (name, val)",
                                         &[&cmd_obj_action_add, &cmd_obj_name, &cmd_obj_ip]);

    let cmd_help_help = TOKEN_STRING_INITIALIZER!(CmdHelpResult, help, "help");

    let cmd_help = cmdline::Inst::new(Some(cmd_help_parsed), None, "show help", &[&cmd_help_help]);

    cmdline::new(&[&cmd_obj_del_show, &cmd_obj_add, &cmd_help])
        .open_stdin("example> ")
        .interact();
}

fn main() {
    let args: Vec<String> = env::args().collect();

    eal::init(&args).expect("Cannot init EAL");

    handle_commands()
}
