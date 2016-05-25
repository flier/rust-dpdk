use rte::*;

use std::os::raw::c_void;

struct CmdGetParams {
    cmd: cmdline::FixedStr,
}

impl CmdGetParams {
    fn quit(&mut self, cl: &cmdline::RawCmdline, _: Option<*mut c_void>) {
        cl.quit();
    }

    fn stats(&mut self, cl: &cmdline::RawCmdline, _: Option<*mut c_void>) {}

    fn drvinfo(&mut self, cl: &cmdline::RawCmdline, _: Option<*mut c_void>) {
        for dev in ethdev::devices() {
            let info = dev.info();

            cl.print(&format!("Port {} driver: {} (ver: {})\n",
                              dev.portid(),
                              info.driver_name(),
                              eal::version()));
        }
    }

    fn link(&mut self, cl: &cmdline::RawCmdline, _: Option<*mut c_void>) {}
}

pub fn main() {
    // Parameter-less commands
    let pcmd_quit_token_cmd = TOKEN_STRING_INITIALIZER!(CmdGetParams, cmd, "quit");
    let pcmd_stats_token_cmd = TOKEN_STRING_INITIALIZER!(CmdGetParams, cmd, "stats");
    let pcmd_drvinfo_token_cmd = TOKEN_STRING_INITIALIZER!(CmdGetParams, cmd, "drvinfo");
    let pcmd_link_token_cmd = TOKEN_STRING_INITIALIZER!(CmdGetParams, cmd, "link");

    let pcmd_quit = cmdline::inst(CmdGetParams::quit,
                                  None,
                                  "quit\n     Exit program",
                                  &[&pcmd_quit_token_cmd]);
    let pcmd_stats = cmdline::inst(CmdGetParams::stats,
                                   None,
                                   "stats\n     Print stats",
                                   &[&pcmd_stats_token_cmd]);
    let pcmd_drvinfo = cmdline::inst(CmdGetParams::drvinfo,
                                     None,
                                     "drvinfo\n     Print driver info",
                                     &[&pcmd_drvinfo_token_cmd]);
    let pcmd_link = cmdline::inst(CmdGetParams::quit,
                                  None,
                                  "link\n     Print port link states",
                                  &[&pcmd_link_token_cmd]);

    let cmds = &[&pcmd_quit, &pcmd_stats, &pcmd_drvinfo, &pcmd_link];

    cmdline::new(cmds)
        .open_stdin("EthApp> ")
        .interact();
}
