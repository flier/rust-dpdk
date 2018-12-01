use std::os::raw::c_void;

use rte::cmdline::*;
use rte::ethdev::{EthDevice, EthDeviceInfo};
use rte::{self, *};

use ethtool::*;

struct CmdGetParams {
    cmd: FixedStr,
}

impl CmdGetParams {
    fn quit(&mut self, cl: &CmdLine, _: Option<&c_void>) {
        debug!("execute `{}` command", self.cmd);

        cl.quit();
    }

    fn drvinfo(&mut self, cl: &CmdLine, _: Option<&c_void>) {
        debug!("execute `{}` command", self.cmd);

        for dev in ethdev::devices() {
            let info = dev.info();

            cl.println(format!(
                "Port {} driver: {} (ver: {})",
                dev.portid(),
                info.driver_name(),
                rte::version()
            )).unwrap();
        }
    }

    fn link(&mut self, cl: &CmdLine, _: Option<&c_void>) {
        debug!("execute `{}` command", self.cmd);

        for dev in ethdev::devices().filter(|dev| dev.is_valid()) {
            let link = dev.link();

            if link.up {
                cl.println(format!(
                    "Port {} Link Up (speed {} Mbps, {})",
                    dev.portid(),
                    link.speed,
                    if link.duplex {
                        "full-duplex"
                    } else {
                        "half-duplex"
                    }
                )).unwrap();
            } else {
                cl.println(format!("Port {} Link Down", dev.portid()))
                    .unwrap();
            }
        }
    }
}

struct CmdIntParams {
    cmd: FixedStr,
    port: u16,
}

impl CmdIntParams {
    fn dev(&self) -> ethdev::PortId {
        self.port as ethdev::PortId
    }

    fn open(&mut self, cl: &CmdLine, app_cfg: Option<&AppConfig>) {
        debug!("execute `{}` command for port {}", self.cmd, self.port);

        let res = app_cfg.unwrap().lock_port(self.dev(), |app_port, dev| {
            dev.stop();

            if let Err(err) = dev.start() {
                Err(format!(
                    "Error: failed to start port {}, {}",
                    self.port, err
                ))
            } else {
                app_port.port_active = true;

                Ok(format!("port {} started", self.port))
            }
        });

        cl.println(res.unwrap_or_else(|err| format!("Error: {}", err)))
            .unwrap();
    }

    fn stop(&mut self, cl: &CmdLine, app_cfg: Option<&AppConfig>) {
        debug!("execute `{}` command for port {}", self.cmd, self.port);

        let res = app_cfg.unwrap().lock_port(self.dev(), |app_port, dev| {
            if !dev.is_up() {
                Err(format!("Port {} already stopped", self.port))
            } else {
                dev.stop();

                app_port.port_active = false;

                Ok(format!("port {} stopped", self.port))
            }
        });

        cl.println(res.unwrap_or_else(|err| format!("Error: {}", err)))
            .unwrap();
    }

    fn rxmode(&mut self, cl: &CmdLine, _: Option<&c_void>) {
        debug!("execute `{}` command for port {}", self.cmd, self.port);

        let dev = self.dev();

        if !dev.is_valid() {
            cl.println(format!("Error: port {} is invalid", self.port))
                .unwrap();
        } else {
            // // Set VF vf_rx_mode, VF unsupport status is discard
            // for vf in 0..dev.info().max_vfs {
            //     if let Err(err) = dev.set_vf_rxmode(vf, ethdev::ETH_VMDQ_ACCEPT_UNTAG, false) {
            //         cl.println(format!(
            //             "Error: failed to set VF rx mode for port {}, {}",
            //             self.port, err
            //         )).unwrap();
            //     }
            // }

            // Enable Rx vlan filter, VF unspport status is discard
            if let Err(err) = dev.set_vlan_offload(ethdev::EthVlanOffloadMode::ETH_VLAN_FILTER_MASK)
            {
                cl.println(format!(
                    "Error: failed to set VLAN offload mode for port {}, {}",
                    self.port, err
                )).unwrap();
            }
        }
    }

    fn portstats(&mut self, cl: &CmdLine, _: Option<&c_void>) {
        debug!("execute `{}` command for port {}", self.cmd, self.port);

        let dev = self.dev();

        cl.println(if !dev.is_valid() {
            format!("Error: port {} is invalid", self.port)
        } else {
            match dev.stats() {
                Ok(stats) => format!(
                    "Port {} stats\n   In: {} ({} bytes)\n  Out: {} ({} bytes)\n  \
                     Err: {}",
                    self.port,
                    stats.ipackets,
                    stats.ibytes,
                    stats.opackets,
                    stats.obytes,
                    stats.ierrors + stats.oerrors
                ),
                Err(err) => format!(
                    "Error: port {} fail to fetch statistics, {}",
                    self.port, err
                ),
            }
        }).unwrap();
    }
}

struct CmdIntMtuParams {
    cmd: FixedStr,
    port: u16,
    mtu: u16,
}

impl CmdIntMtuParams {
    fn dev(&self) -> ethdev::PortId {
        self.port as ethdev::PortId
    }

    fn mtu_list(&mut self, cl: &CmdLine, app_cfg: Option<&AppConfig>) {
        debug!("execute list `{}` command for port {}", self.cmd, self.port);

        for portid in 0..app_cfg.unwrap().ports.len() {
            let dev = portid as ethdev::PortId;

            cl.println(format!("Port {} MTU: {}", portid, dev.mtu().unwrap()))
                .unwrap();
        }
    }

    fn mtu_get(&mut self, cl: &CmdLine, _: Option<&c_void>) {
        debug!("execute get `{}` command for port {}", self.cmd, self.port);

        let dev = self.dev();

        cl.println(if !dev.is_valid() {
            format!("Error: port {} is invalid", self.port)
        } else {
            format!("Port {} MTU: {}", self.port, dev.mtu().unwrap())
        }).unwrap();
    }

    fn mtu_set(&mut self, cl: &CmdLine, _: Option<&c_void>) {
        debug!("execute set `{}` command for port {}", self.cmd, self.port);

        let dev = self.dev();

        cl.println(if let Err(err) = dev.set_mtu(self.mtu) {
            format!(
                "Error: Fail to change mac address of port {}, {}",
                self.port, err
            )
        } else {
            format!("Port {} MTU was changed to {}", self.port, self.mtu)
        }).unwrap();
    }
}

struct CmdIntMacParams {
    cmd: FixedStr,
    port: u16,
    mac: EtherAddr,
}

impl CmdIntMacParams {
    fn dev(&self) -> ethdev::PortId {
        self.port as ethdev::PortId
    }

    fn list(&mut self, cl: &CmdLine, app_cfg: Option<&AppConfig>) {
        debug!("execute list `{}` command for port {}", self.cmd, self.port);

        for portid in 0..app_cfg.unwrap().ports.len() {
            let dev = portid as ethdev::PortId;

            cl.println(format!("Port {} MAC Address: {}", portid, dev.mac_addr()))
                .unwrap();
        }
    }

    fn get(&mut self, cl: &CmdLine, _: Option<&c_void>) {
        debug!("execute get `{}` command for port {}", self.cmd, self.port);

        let dev = self.dev();

        cl.println(if !dev.is_valid() {
            format!("Error: port {} is invalid", self.port)
        } else {
            format!("Port {} MAC Address: {}", self.port, dev.mac_addr())
        }).unwrap();
    }

    fn set(&mut self, cl: &CmdLine, app_cfg: Option<&AppConfig>) {
        debug!("execute set `{}` command for port {}", self.cmd, self.port);

        let res = app_cfg.unwrap().lock_port(self.dev(), |app_port, dev| {
            if let Err(err) = dev.set_mac_addr(&self.mac) {
                Err(format!(
                    "Fail to change mac address of port {}, {}",
                    self.port, err
                ))
            } else {
                app_port.port_dirty = true;

                Ok(format!(
                    "Port {} mac address was changed to {}",
                    self.port, self.mac
                ))
            }
        });

        cl.println(res.unwrap_or_else(|err| format!("Error: {}", err)))
            .unwrap();
    }

    fn validate(&mut self, cl: &CmdLine, _: Option<&c_void>) {
        debug!("execute `{}` command for port {}", self.cmd, self.port);

        cl.println(format!(
            "MAC address {} is {}",
            self.mac,
            if self.mac.is_valid() {
                "unicast"
            } else {
                "not unicast"
            }
        )).unwrap();
    }
}

struct CmdVlanParams {
    cmd: FixedStr,
    port: u16,
    mode: FixedStr,
    vlan_id: u16,
}

impl CmdVlanParams {
    fn dev(&self) -> ethdev::PortId {
        self.port as ethdev::PortId
    }

    fn change(&mut self, cl: &CmdLine, _: Option<&c_void>) {
        debug!("execute `{}` command for port {}", self.cmd, self.port);

        let dev = self.dev();

        cl.println(if !dev.is_valid() {
            format!("Error: port {} is invalid", self.port)
        } else {
            match self.mode.to_str() {
                "add" => match dev.set_vlan_filter(self.vlan_id, true) {
                    Ok(_) => format!("VLAN vid {} added to port {}", self.vlan_id, self.port),
                    Err(err) => format!(
                        "Error: fail to add VLAN vid {} to port {}, {}",
                        self.vlan_id, self.port, err
                    ),
                },
                "del" => match dev.set_vlan_filter(self.vlan_id, false) {
                    Ok(_) => format!("VLAN vid {} removed from port {}", self.vlan_id, self.port),
                    Err(err) => format!(
                        "Error: fail to remove VLAN vid {} to port {}, {}",
                        self.vlan_id, self.port, err
                    ),
                },
                mode @ _ => format!("Error: Bad mode {}", mode),
            }
        }).unwrap();
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
    let pcmd_mtu_token_cmd = TOKEN_STRING_INITIALIZER!(CmdIntMtuParams, cmd, "mtu");
    let pcmd_intmtu_token_port = TOKEN_NUM_INITIALIZER!(CmdIntMtuParams, port, u16);
    let pcmd_intmtu_token_opt = TOKEN_NUM_INITIALIZER!(CmdIntMtuParams, mtu, u16);

    // Commands taking port id and a MAC address string
    let pcmd_macaddr_token_cmd = TOKEN_STRING_INITIALIZER!(CmdIntMacParams, cmd, "macaddr");
    let pcmd_intmac_token_port = TOKEN_NUM_INITIALIZER!(CmdIntMacParams, port, u16);
    let pcmd_intmac_token_mac = TOKEN_ETHERADDR_INITIALIZER!(CmdIntMacParams, mac);

    // Command taking just a MAC address
    let pcmd_validate_token_cmd = TOKEN_STRING_INITIALIZER!(CmdIntMacParams, cmd, "validate");

    // /* VLAN commands */
    let pcmd_vlan_token_cmd = TOKEN_STRING_INITIALIZER!(CmdVlanParams, cmd, "vlan");
    let pcmd_vlan_token_port = TOKEN_NUM_INITIALIZER!(CmdVlanParams, port, u16);
    let pcmd_vlan_token_mode = TOKEN_STRING_INITIALIZER!(CmdVlanParams, mode, "add#del");
    let pcmd_vlan_token_vlan_id = TOKEN_NUM_INITIALIZER!(CmdVlanParams, vlan_id, u16);

    let pcmd_quit = inst(
        CmdGetParams::quit,
        None,
        "quit\n     Exit program",
        &[&pcmd_quit_token_cmd],
    );

    let pcmd_drvinfo = inst(
        CmdGetParams::drvinfo,
        None,
        "drvinfo\n     Print driver info",
        &[&pcmd_drvinfo_token_cmd],
    );

    let pcmd_link = inst(
        CmdGetParams::link,
        None,
        "link\n     Print port link states",
        &[&pcmd_link_token_cmd],
    );

    let pcmd_open = inst(
        CmdIntParams::open,
        Some(app_cfg),
        "open <port_id>\n     Open port",
        &[&pcmd_open_token_cmd, &pcmd_int_token_port],
    );

    let pcmd_stop = inst(
        CmdIntParams::stop,
        Some(app_cfg),
        "stop <port_id>\n     Stop port",
        &[&pcmd_stop_token_cmd, &pcmd_int_token_port],
    );

    let pcmd_rxmode = inst(
        CmdIntParams::rxmode,
        None,
        "rxmode <port_id>\n     Toggle port Rx mode",
        &[&pcmd_rxmode_token_cmd, &pcmd_int_token_port],
    );

    let pcmd_portstats = inst(
        CmdIntParams::portstats,
        None,
        "portstats <port_id>\n     Print port eth statistics",
        &[&pcmd_portstats_token_cmd, &pcmd_int_token_port],
    );

    let pcmd_mtu_list = inst(
        CmdIntMtuParams::mtu_list,
        Some(app_cfg),
        "mtu\n     List MTU",
        &[&pcmd_mtu_token_cmd],
    );

    let pcmd_mtu_get = inst(
        CmdIntMtuParams::mtu_get,
        None,
        "mtu <port_id>\n     Show MTU",
        &[&pcmd_mtu_token_cmd, &pcmd_intmtu_token_port],
    );

    let pcmd_mtu_set = inst(
        CmdIntMtuParams::mtu_set,
        None,
        "mtu <port_id> <mtu_value>\n     Change MTU",
        &[
            &pcmd_mtu_token_cmd,
            &pcmd_intmtu_token_port,
            &pcmd_intmtu_token_opt,
        ],
    );

    let pcmd_macaddr_list = inst(
        CmdIntMacParams::list,
        Some(app_cfg),
        "macaddr\n     List port MAC address",
        &[&pcmd_macaddr_token_cmd],
    );

    let pcmd_macaddr_get = inst(
        CmdIntMacParams::get,
        None,
        "macaddr <port_id>\n     Get MAC address",
        &[&pcmd_macaddr_token_cmd, &pcmd_intmac_token_port],
    );

    let pcmd_macaddr_set = inst(
        CmdIntMacParams::set,
        Some(app_cfg),
        "macaddr <port_id> <mac_addr>\n     Set MAC address",
        &[
            &pcmd_macaddr_token_cmd,
            &pcmd_intmac_token_port,
            &pcmd_intmac_token_mac,
        ],
    );

    let pcmd_macaddr_validate = inst(
        CmdIntMacParams::validate,
        None,
        "validate <mac_addr>\n     Check that MAC address \
         is valid unicast address",
        &[&pcmd_validate_token_cmd, &pcmd_intmac_token_mac],
    );

    let pcmd_vlan = inst(
        CmdVlanParams::change,
        None,
        "vlan <port_id> <add|del> <vlan_id>\n     Add/remove VLAN id",
        &[
            &pcmd_vlan_token_cmd,
            &pcmd_vlan_token_port,
            &pcmd_vlan_token_mode,
            &pcmd_vlan_token_vlan_id,
        ],
    );

    let cmds = &[
        &pcmd_quit,
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
        &pcmd_macaddr_validate,
        &pcmd_vlan,
    ];

    new(cmds)
        .open_stdin("EthApp> ")
        .expect("fail to open stdin")
        .interact();
}
