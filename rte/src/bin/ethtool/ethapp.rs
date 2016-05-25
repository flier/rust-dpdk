use std::os::raw::c_void;

use rte::*;

use ethtool::*;

struct CmdGetParams {
    cmd: cmdline::FixedStr,
}

impl CmdGetParams {
    fn quit(&mut self, cl: &cmdline::RawCmdline, _: Option<*mut c_void>) {
        debug!("execute `{}` command", cmdline::str(&self.cmd).unwrap());

        cl.quit();
    }

    fn drvinfo(&mut self, cl: &cmdline::RawCmdline, _: Option<*mut c_void>) {
        debug!("execute `{}` command", cmdline::str(&self.cmd).unwrap());

        for dev in ethdev::devices() {
            let info = dev.info();

            cl.print(format!("Port {} driver: {} (ver: {})\n",
                               dev.portid(),
                               info.driver_name(),
                               eal::version()))
                .unwrap();
        }
    }

    fn link(&mut self, cl: &cmdline::RawCmdline, _: Option<*mut c_void>) {
        debug!("execute `{}` command", cmdline::str(&self.cmd).unwrap());

        for dev in ethdev::devices().filter(|dev| dev.is_valid()) {
            let link = dev.link();

            if link.up {
                cl.print(format!("Port {} Link Up (speed {} Mbps, {})\n",
                                   dev.portid(),
                                   link.speed,
                                   if link.duplex {
                                       "full-duplex"
                                   } else {
                                       "half-duplex"
                                   }))
                    .unwrap();
            } else {
                cl.print(format!("Port {} Link Down\n", dev.portid())).unwrap();
            }
        }
    }
}

struct CmdIntParams {
    cmd: cmdline::FixedStr,
    port: u16,
}

impl CmdIntParams {
    fn open(&mut self, cl: &cmdline::RawCmdline, app_cfg: Option<*mut AppConfig>) {
        debug!("execute `{}` command for port {}",
               cmdline::str(&self.cmd).unwrap(),
               self.port);

        match unsafe { (*app_cfg.unwrap()).ports.iter().nth(self.port as usize) } {
            Some(mutex) => {
                let dev = ethdev::EthDevice::from(self.port as u8);

                if !dev.is_valid() {
                    cl.print(format!("Error: port {} is invalid\n", self.port)).unwrap();
                } else if let Ok(mut guard) = mutex.lock() {
                    let app_port: &mut AppPort = &mut *guard;

                    dev.stop();

                    if let Err(err) = dev.start() {
                        cl.print(format!("Error: failed to start port {}, {}", self.port, err))
                            .unwrap();
                    } else {
                        app_port.port_active = true;

                        info!("port {} started", self.port);
                    }
                }
            }
            _ => {
                cl.print(format!("Error: port number {} is invalid\n", self.port)).unwrap();
            }
        }
    }

    fn stop(&mut self, cl: &cmdline::RawCmdline, app_cfg: Option<*mut AppConfig>) {
        debug!("execute `{}` command for port {}",
               cmdline::str(&self.cmd).unwrap(),
               self.port);

        match unsafe { (*app_cfg.unwrap()).ports.iter().nth(self.port as usize) } {
            Some(mutex) => {
                let dev = ethdev::EthDevice::from(self.port as u8);

                if !dev.is_valid() {
                    cl.print(format!("Error: port {} is invalid\n", self.port)).unwrap();
                } else if !dev.is_up() {
                    cl.print(format!("Port {} already stopped\n", self.port)).unwrap();
                } else if let Ok(mut guard) = mutex.lock() {
                    let app_port: &mut AppPort = &mut *guard;

                    dev.stop();

                    app_port.port_active = false;

                    info!("port {} stopped", self.port);
                }
            }
            _ => {
                cl.print(format!("Error: port number {} is invalid\n", self.port)).unwrap();
            }
        }
    }

    fn rxmode(&mut self, cl: &cmdline::RawCmdline, _: Option<*mut c_void>) {
        debug!("execute `{}` command for port {}",
               cmdline::str(&self.cmd).unwrap(),
               self.port);

        let dev = ethdev::EthDevice::from(self.port as u8);

        if !dev.is_valid() {
            cl.print(format!("Error: port {} is invalid\n", self.port)).unwrap();
        } else {
            // Set VF vf_rx_mode, VF unsupport status is discard
            for vf in 0..(*dev.info()).max_vfs {
                if let Err(err) = dev.set_vf_rxmode(vf, ethdev::ETH_VMDQ_ACCEPT_UNTAG, false) {
                    cl.print(format!("Error: failed to set VF rx mode for port {}, {}",
                                       self.port,
                                       err))
                        .unwrap();
                }
            }

            // Enable Rx vlan filter, VF unspport status is discard
            if let Err(err) = dev.set_vlan_offload(ethdev::ETH_VLAN_FILTER_MASK) {
                cl.print(format!("Error: failed to set VLAN offload mode for port {}, {}",
                                   self.port,
                                   err))
                    .unwrap();
            }
        }
    }

    fn portstats(&mut self, cl: &cmdline::RawCmdline, _: Option<*mut c_void>) {
        debug!("execute `{}` command for port {}",
               cmdline::str(&self.cmd).unwrap(),
               self.port);
    }
}

pub fn main(app_cfg: &mut AppConfig) {
    // Parameter-less commands
    let pcmd_quit_token_cmd = TOKEN_STRING_INITIALIZER!(CmdGetParams, cmd, "quit");
    let pcmd_drvinfo_token_cmd = TOKEN_STRING_INITIALIZER!(CmdGetParams, cmd, "drvinfo");
    let pcmd_link_token_cmd = TOKEN_STRING_INITIALIZER!(CmdGetParams, cmd, "link");

    // Commands taking just port id
    let pcmd_open_token_cmd = TOKEN_STRING_INITIALIZER!(CmdIntParams, cmd, "open");
    let pcmd_stop_token_cmd = TOKEN_STRING_INITIALIZER!(CmdIntParams, cmd, "stop");
    let pcmd_rxmode_token_cmd = TOKEN_STRING_INITIALIZER!(CmdIntParams, cmd, "rxmode");
    let pcmd_portstats_token_cmd = TOKEN_STRING_INITIALIZER!(CmdIntParams, cmd, "portstats");

    let pcmd_int_token_port = TOKEN_NUM_INITIALIZER!(CmdIntParams, port, u16);

    let pcmd_quit = cmdline::inst(CmdGetParams::quit,
                                  None,
                                  "quit\n     Exit program",
                                  &[&pcmd_quit_token_cmd]);
    let pcmd_drvinfo = cmdline::inst(CmdGetParams::drvinfo,
                                     None,
                                     "drvinfo\n     Print driver info",
                                     &[&pcmd_drvinfo_token_cmd]);
    let pcmd_link = cmdline::inst(CmdGetParams::link,
                                  None,
                                  "link\n     Print port link states",
                                  &[&pcmd_link_token_cmd]);

    let pcmd_open = cmdline::inst(CmdIntParams::open,
                                  Some(app_cfg),
                                  "open <port_id>\n     Open port",
                                  &[&pcmd_open_token_cmd, &pcmd_int_token_port]);
    let pcmd_stop = cmdline::inst(CmdIntParams::stop,
                                  Some(app_cfg),
                                  "stop <port_id>\n     Stop port",
                                  &[&pcmd_stop_token_cmd, &pcmd_int_token_port]);
    let pcmd_rxmode = cmdline::inst(CmdIntParams::rxmode,
                                    None,
                                    "rxmode <port_id>\n     Toggle port Rx mode",
                                    &[&pcmd_rxmode_token_cmd, &pcmd_int_token_port]);
    let pcmd_portstats = cmdline::inst(CmdIntParams::portstats,
                                       None,
                                       "portstats <port_id>\n     Print port eth statistics",
                                       &[&pcmd_portstats_token_cmd, &pcmd_int_token_port]);

    let cmds = &[&pcmd_quit,
                 &pcmd_drvinfo,
                 &pcmd_link,
                 &pcmd_open,
                 &pcmd_stop,
                 &pcmd_rxmode,
                 &pcmd_portstats];

    cmdline::new(cmds)
        .open_stdin("EthApp> ")
        .interact();
}
