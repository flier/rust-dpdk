use rte::*;

use std::os::raw::c_void;

struct CmdGetParams {
    cmd: cmdline::FixedStr,
}

impl CmdGetParams {
    fn quit(&mut self, cl: &cmdline::RawCmdline, _: Option<*mut c_void>) {
        cl.quit();
    }

    fn drvinfo(&mut self, cl: &cmdline::RawCmdline, _: Option<*mut c_void>) {
        for dev in ethdev::devices() {
            let info = dev.info();

            cl.print(&format!("Port {} driver: {} (ver: {})\n",
                                dev.portid(),
                                info.driver_name(),
                                eal::version()))
                .unwrap();
        }
    }

    fn link(&mut self, cl: &cmdline::RawCmdline, _: Option<*mut c_void>) {
        for dev in ethdev::devices().filter(|dev| dev.is_attached()) {
            let link = dev.link();

            if link.up {
                cl.print(&format!("Port {} Link Up (speed {} Mbps, {})\n",
                                    dev.portid(),
                                    link.speed,
                                    if link.duplex {
                                        "full-duplex"
                                    } else {
                                        "half-duplex"
                                    }))
                    .unwrap();
            } else {
                cl.print(&format!("Port {} Link Down\n", dev.portid())).unwrap();
            }
        }
    }
}

pub fn main() {
    // Parameter-less commands
    let pcmd_quit_token_cmd = TOKEN_STRING_INITIALIZER!(CmdGetParams, cmd, "quit");
    let pcmd_drvinfo_token_cmd = TOKEN_STRING_INITIALIZER!(CmdGetParams, cmd, "drvinfo");
    let pcmd_link_token_cmd = TOKEN_STRING_INITIALIZER!(CmdGetParams, cmd, "link");

    let pcmd_quit = cmdline::inst(CmdGetParams::quit,
                                  None,
                                  "quit\n     Exit program",
                                  &[&pcmd_quit_token_cmd]);
    let pcmd_drvinfo = cmdline::inst(CmdGetParams::drvinfo,
                                     None,
                                     "drvinfo\n     Print driver info",
                                     &[&pcmd_drvinfo_token_cmd]);
    let pcmd_link = cmdline::inst(CmdGetParams::quit,
                                  None,
                                  "link\n     Print port link states",
                                  &[&pcmd_link_token_cmd]);

    let cmds = &[&pcmd_quit, &pcmd_drvinfo, &pcmd_link];

    cmdline::new(cmds)
        .open_stdin("EthApp> ")
        .interact();
}
