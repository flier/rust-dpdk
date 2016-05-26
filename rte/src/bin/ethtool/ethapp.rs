use std::os::raw::c_void;

use rte::*;

use ethtool::*;

struct CmdGetParams {
    cmd: cmdline::FixedStr,
}

impl CmdGetParams {
    fn cmd(&self) -> &str {
        cmdline::str(&self.cmd).unwrap()
    }

    fn quit(&mut self, cl: &cmdline::RawCmdline, _: Option<&c_void>) {
        debug!("execute `{}` command", self.cmd());

        cl.quit();
    }

    fn drvinfo(&mut self, cl: &cmdline::RawCmdline, _: Option<&c_void>) {
        debug!("execute `{}` command", self.cmd());

        for dev in ethdev::devices() {
            let info = dev.info();

            cl.println(format!("Port {} driver: {} (ver: {})",
                                 dev.portid(),
                                 info.driver_name(),
                                 eal::version()))
                .unwrap();
        }
    }

    fn link(&mut self, cl: &cmdline::RawCmdline, _: Option<&c_void>) {
        debug!("execute `{}` command", self.cmd());

        for dev in ethdev::devices().filter(|dev| dev.is_valid()) {
            let link = dev.link();

            if link.up {
                cl.println(format!("Port {} Link Up (speed {} Mbps, {})",
                                     dev.portid(),
                                     link.speed,
                                     if link.duplex {
                                         "full-duplex"
                                     } else {
                                         "half-duplex"
                                     }))
                    .unwrap();
            } else {
                cl.println(format!("Port {} Link Down", dev.portid())).unwrap();
            }
        }
    }
}

struct CmdIntParams {
    cmd: cmdline::FixedStr,
    port: u16,
}

impl CmdIntParams {
    fn cmd(&self) -> &str {
        cmdline::str(&self.cmd).unwrap()
    }

    fn open(&mut self, cl: &cmdline::RawCmdline, app_cfg: Option<&AppConfig>) {
        debug!("execute `{}` command for port {}", self.cmd(), self.port);

        let res = app_cfg.unwrap().lock_port(self.port as u8, |app_port, dev| {
            dev.stop();

            if let Err(err) = dev.start() {
                Err(format!("Error: failed to start port {}, {}", self.port, err))
            } else {
                app_port.port_active = true;

                Ok(format!("port {} started", self.port))
            }
        });

        cl.println(res.unwrap_or_else(|err| format!("Error: {}", err))).unwrap();
    }

    fn stop(&mut self, cl: &cmdline::RawCmdline, app_cfg: Option<&AppConfig>) {
        debug!("execute `{}` command for port {}", self.cmd(), self.port);

        let res = app_cfg.unwrap().lock_port(self.port as u8, |app_port, dev| {
            if !dev.is_up() {
                Err(format!("Port {} already stopped", self.port))
            } else {
                dev.stop();

                app_port.port_active = false;

                Ok(format!("port {} stopped", self.port))
            }
        });

        cl.println(res.unwrap_or_else(|err| format!("Error: {}", err))).unwrap();
    }

    fn rxmode(&mut self, cl: &cmdline::RawCmdline, _: Option<&c_void>) {
        debug!("execute `{}` command for port {}", self.cmd(), self.port);

        let dev = ethdev::EthDevice::from(self.port as u8);

        if !dev.is_valid() {
            cl.println(format!("Error: port {} is invalid", self.port)).unwrap();
        } else {
            // Set VF vf_rx_mode, VF unsupport status is discard
            for vf in 0..(*dev.info()).max_vfs {
                if let Err(err) = dev.set_vf_rxmode(vf, ethdev::ETH_VMDQ_ACCEPT_UNTAG, false) {
                    cl.println(format!("Error: failed to set VF rx mode for port {}, {}",
                                         self.port,
                                         err))
                        .unwrap();
                }
            }

            // Enable Rx vlan filter, VF unspport status is discard
            if let Err(err) = dev.set_vlan_offload(ethdev::ETH_VLAN_FILTER_MASK) {
                cl.println(format!("Error: failed to set VLAN offload mode for port {}, {}",
                                     self.port,
                                     err))
                    .unwrap();
            }
        }
    }

    fn portstats(&mut self, cl: &cmdline::RawCmdline, _: Option<&c_void>) {
        debug!("execute `{}` command for port {}", self.cmd(), self.port);

        let dev = ethdev::EthDevice::from(self.port as u8);

        cl.println(if !dev.is_valid() {
                format!("Error: port {} is invalid", self.port)
            } else {
                match dev.stats() {
                    Ok(stats) => {
                        format!("Port {} stats\n   In: {} ({} bytes)\n  Out: {} ({} bytes)\n  \
                                 Err: {}",
                                self.port,
                                stats.ipackets,
                                stats.ibytes,
                                stats.opackets,
                                stats.obytes,
                                stats.ierrors + stats.oerrors)
                    }
                    Err(err) => {
                        format!("Error: port {} fail to fetch statistics, {}",
                                self.port,
                                err)
                    }
                }
            })
            .unwrap()
    }
}

struct CmdIntStrParams {
    cmd: cmdline::FixedStr,
    port: u16,
    opt: cmdline::FixedStr,
}

impl CmdIntStrParams {
    fn cmd(&self) -> &str {
        cmdline::str(&self.cmd).unwrap()
    }

    fn opt(&self) -> &str {
        cmdline::str(&self.opt).unwrap()
    }

    fn mtu_list(&mut self, cl: &cmdline::RawCmdline, app_cfg: Option<&AppConfig>) {
        debug!("execute list `{}` command for port {}",
               self.cmd(),
               self.port);

        for portid in 0..app_cfg.unwrap().ports.len() {
            let dev = ethdev::EthDevice::from(portid as u8);

            cl.println(format!("Port {} MTU: {}", portid, dev.mtu().unwrap()))
                .unwrap();
        }
    }

    fn mtu_get(&mut self, cl: &cmdline::RawCmdline, _: Option<&c_void>) {
        debug!("execute get `{}` command for port {}",
               self.cmd(),
               self.port);

        let dev = ethdev::EthDevice::from(self.port as u8);

        cl.println(if !dev.is_valid() {
                format!("Error: port {} is invalid", self.port)
            } else {
                format!("Port {} MTU: {}", self.port, dev.mtu().unwrap())
            })
            .unwrap()
    }

    fn mtu_set(&mut self, cl: &cmdline::RawCmdline, _: Option<&c_void>) {
        debug!("execute set `{}` command for port {}",
               self.cmd(),
               self.port);

        let dev = ethdev::EthDevice::from(self.port as u8);

        cl.println(match self.opt().parse::<u32>() {
                Ok(mtu) => {
                    if let Err(err) = dev.set_mtu(mtu as u16) {
                        format!("Error: Fail to change mac address of port {}, {}",
                                self.port,
                                err)
                    } else {
                        format!("Port {} MTU was changed to {}", self.port, mtu)
                    }
                }
                Err(err) => format!("Error: invalid MTU number {}, {}", self.opt(), err),
            })
            .unwrap()
    }
}

struct CmdIntMacParams {
    cmd: cmdline::FixedStr,
    port: u16,
    mac: cmdline::EtherAddr,
}

impl CmdIntMacParams {
    fn cmd(&self) -> &str {
        cmdline::str(&self.cmd).unwrap()
    }

    fn mac_addr(&self) -> ether::EtherAddr {
        cmdline::etheraddr(&self.mac)
    }

    fn list(&mut self, cl: &cmdline::RawCmdline, app_cfg: Option<&AppConfig>) {
        debug!("execute list `{}` command for port {}",
               self.cmd(),
               self.port);

        for portid in 0..app_cfg.unwrap().ports.len() {
            let dev = ethdev::EthDevice::from(portid as u8);

            cl.println(format!("Port {} MAC Address: {}", portid, dev.mac_addr()))
                .unwrap();
        }
    }

    fn get(&mut self, cl: &cmdline::RawCmdline, _: Option<&c_void>) {
        debug!("execute get `{}` command for port {}",
               self.cmd(),
               self.port);

        let dev = ethdev::EthDevice::from(self.port as u8);

        cl.println(if !dev.is_valid() {
                format!("Error: port {} is invalid", self.port)
            } else {
                format!("Port {} MAC Address: {}", self.port, dev.mac_addr())
            })
            .unwrap()
    }

    fn set(&mut self, cl: &cmdline::RawCmdline, app_cfg: Option<&AppConfig>) {
        debug!("execute set `{}` command for port {}",
               self.cmd(),
               self.port);

        let mac_addr = self.mac_addr();

        let res = app_cfg.unwrap().lock_port(self.port as u8, |app_port, dev| {
            if let Err(err) = dev.set_mac_addr(&mac_addr) {
                Err(format!("Fail to change mac address of port {}, {}", self.port, err))
            } else {
                app_port.port_dirty = true;

                Ok(format!("Port {} mac address was changed to {}", self.port, mac_addr))
            }
        });

        cl.println(res.unwrap_or_else(|err| format!("Error: {}", err))).unwrap();
    }

    fn validate(&mut self, cl: &cmdline::RawCmdline, _: Option<&c_void>) {
        debug!("execute `{}` command for port {}", self.cmd(), self.port);

        let mac_addr = self.mac_addr();

        cl.println(format!("MAC address {} is {}",
                             mac_addr,
                             if mac_addr.is_valid() {
                                 "unicast"
                             } else {
                                 "not unicast"
                             }))
            .unwrap()
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

    // Commands taking port id and string
    let pcmd_mtu_token_cmd = TOKEN_STRING_INITIALIZER!(CmdIntStrParams, cmd, "mtu");

    let pcmd_intstr_token_port = TOKEN_NUM_INITIALIZER!(CmdIntStrParams, port, u16);
    let pcmd_intstr_token_opt = TOKEN_STRING_INITIALIZER!(CmdIntStrParams, opt);

    // Commands taking port id and a MAC address string
    let pcmd_macaddr_token_cmd = TOKEN_STRING_INITIALIZER!(CmdIntMacParams, cmd, "macaddr");
    let pcmd_intmac_token_port = TOKEN_NUM_INITIALIZER!(CmdIntMacParams, port, u16);
    let pcmd_intmac_token_mac = TOKEN_ETHERADDR_INITIALIZER!(CmdIntMacParams, mac);

    // Command taking just a MAC address
    let pcmd_validate_token_cmd = TOKEN_STRING_INITIALIZER!(CmdIntMacParams, cmd, "validate");

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

    let pcmd_mtu_list = cmdline::inst(CmdIntStrParams::mtu_list,
                                      Some(app_cfg),
                                      "mtu\n     List MTU",
                                      &[&pcmd_mtu_token_cmd]);

    let pcmd_mtu_get = cmdline::inst(CmdIntStrParams::mtu_get,
                                     None,
                                     "mtu <port_id>\n     Show MTU",
                                     &[&pcmd_mtu_token_cmd, &pcmd_intstr_token_port]);

    let pcmd_mtu_set =
        cmdline::inst(CmdIntStrParams::mtu_set,
                      None,
                      "mtu <port_id> <mtu_value>\n     Change MTU",
                      &[&pcmd_mtu_token_cmd, &pcmd_intstr_token_port, &pcmd_intstr_token_opt]);


    let pcmd_macaddr_list = cmdline::inst(CmdIntMacParams::list,
                                          Some(app_cfg),
                                          "macaddr\n     List port MAC address",
                                          &[&pcmd_macaddr_token_cmd]);

    let pcmd_macaddr_get = cmdline::inst(CmdIntMacParams::get,
                                         None,
                                         "macaddr <port_id>\n     Get MAC address",
                                         &[&pcmd_macaddr_token_cmd, &pcmd_intmac_token_port]);

    let pcmd_macaddr_set =
        cmdline::inst(CmdIntMacParams::set,
                      Some(app_cfg),
                      "macaddr <port_id> <mac_addr>\n     Set MAC address",
                      &[&pcmd_macaddr_token_cmd, &pcmd_intmac_token_port, &pcmd_intmac_token_mac]);

    let pcmd_macaddr_validate = cmdline::inst(CmdIntMacParams::validate,
                                              None,
                                              "validate <mac_addr>\n     Check that MAC address \
                                               is valid unicast address",
                                              &[&pcmd_validate_token_cmd, &pcmd_intmac_token_mac]);

    let cmds = &[&pcmd_quit,
                 &pcmd_drvinfo,
                 &pcmd_link,
                 &pcmd_open,
                 &pcmd_stop,
                 &pcmd_rxmode,
                 &pcmd_portstats,
                 &pcmd_mtu_list,
                 &pcmd_mtu_get,
                 &pcmd_mtu_set,
                 &pcmd_macaddr_list,
                 &pcmd_macaddr_get,
                 &pcmd_macaddr_set,
                 &pcmd_macaddr_validate];

    cmdline::new(cmds)
        .open_stdin("EthApp> ")
        .interact();
}
