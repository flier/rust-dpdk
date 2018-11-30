#[macro_use]
extern crate log;
extern crate libc;
extern crate pretty_env_logger;
extern crate rte;

mod ethapp;
mod ethtool;

use std::env;

use rte::ethdev::EthDevice;
use rte::memory::AsMutRef;
use rte::*;

use ethtool::*;

const PORT_RX_QUEUE_SIZE: u16 = 128;
const PORT_TX_QUEUE_SIZE: u16 = 256;

const PKTPOOL_EXTRA_SIZE: u16 = 512;
const PKTPOOL_CACHE: u32 = 32;

const EXIT_FAILURE: i32 = -1;

fn setup_ports(app_cfg: &mut AppConfig) {
    let port_conf = ethdev::EthConf::default();

    for (portid, mutex) in app_cfg.ports.iter().enumerate() {
        if let Ok(mut guard) = mutex.lock() {
            let app_port: &mut AppPort = &mut *guard;

            let dev = portid as ethdev::PortId;
            let dev_info = dev.info();

            let size_pktpool =
                dev_info.rx_desc_lim.nb_max + dev_info.tx_desc_lim.nb_max + PKTPOOL_EXTRA_SIZE;

            app_port.pkt_pool = mbuf::pktmbuf_pool_create(
                &format!("pkt_pool_{}", portid),
                size_pktpool as u32,
                PKTPOOL_CACHE,
                0,
                mbuf::RTE_MBUF_DEFAULT_BUF_SIZE,
                eal::socket_id(),
            ).expect("create mbuf pool failed");

            println!("Init port {}..\n", portid);

            app_port.mac_addr = dev.mac_addr();
            app_port.port_active = true;
            app_port.port_id = portid as u8;

            dev.configure(1, 1, &port_conf)
                .expect(&format!("fail to configure device: port={}", portid));

            // init one RX queue
            dev.rx_queue_setup(
                0,
                PORT_RX_QUEUE_SIZE,
                None,
                app_port.pkt_pool.as_mut_ref().unwrap(),
            ).expect(&format!("fail to setup device rx queue: port={}", portid));

            // init one TX queue on each port
            dev.tx_queue_setup(0, PORT_TX_QUEUE_SIZE, None)
                .expect(&format!("fail to setup device tx queue: port={}", portid));

            // Start device
            dev.start()
                .expect(&format!("fail to start device: port={}", portid));

            dev.promiscuous_enable();
        }
    }
}

fn process_frame(mac_addr: &ether::EtherAddr, frame: mbuf::RawMbufPtr) {
    if let Some(ether_hdr) = pktmbuf_mtod!(frame, *mut ether::EtherHdr).as_mut_ref() {
        ether::EtherAddr::copy(
            &ether_hdr.s_addr.addr_bytes,
            &mut ether_hdr.d_addr.addr_bytes,
        );
        ether::EtherAddr::copy(&mac_addr, &mut ether_hdr.s_addr.addr_bytes);
    }
}

fn slave_main(app_cfg: Option<&AppConfig>) -> i32 {
    let app_cfg = app_cfg.unwrap();

    while !app_cfg.exit_now {
        for (portid, mutex) in app_cfg.ports.iter().enumerate() {
            // Check that port is active and unlocked
            if let Ok(mut guard) = mutex.try_lock() {
                let app_port: &mut AppPort = &mut *guard;

                if !app_port.port_active {
                    continue;
                }

                let dev = portid as ethdev::PortId;

                // MAC address was updated
                if app_port.port_dirty {
                    app_port.mac_addr = dev.mac_addr();
                    app_port.port_dirty = false;
                }

                let txq = &mut app_port.txq;

                // Incoming frames
                let cnt_recv_frames = dev.rx_burst(0, &mut txq.buf_frames[txq.cnt_unsent..]);

                if cnt_recv_frames > 0 {
                    let frames = &txq.buf_frames[txq.cnt_unsent..txq.cnt_unsent + cnt_recv_frames];
                    for frame in frames {
                        process_frame(&app_port.mac_addr, *frame);
                    }

                    txq.cnt_unsent += cnt_recv_frames
                }

                // Outgoing frames
                if txq.cnt_unsent > 0 {
                    let cnt_sent = dev.tx_burst(0, &mut txq.buf_frames[..txq.cnt_unsent]);

                    for i in cnt_sent..txq.cnt_unsent {
                        txq.buf_frames[i - cnt_sent] = txq.buf_frames[i];
                    }
                }
            }
        }
    }

    0
}

fn main() {
    pretty_env_logger::init();

    let args: Vec<String> = env::args().collect();

    // Init runtime enviornment
    eal::init(&args).expect("Cannot init EAL");

    let cnt_ports = match ethdev::count() {
        0 => {
            eal::exit(EXIT_FAILURE, "No available NIC ports!\n");

            0
        }
        ports @ 1...MAX_PORTS => ports,
        ports @ _ => {
            println!("Using only {} of {} ports", MAX_PORTS, ports);

            MAX_PORTS
        }
    } as u32;

    println!("Number of NICs: {}", cnt_ports);

    let mut app_cfg = AppConfig::new(cnt_ports);

    if lcore::count() < 2 {
        eal::exit(EXIT_FAILURE, "No available slave core!\n");
    }

    setup_ports(&mut app_cfg);

    // Assume there is an available slave..
    let lcore_id = lcore::next(lcore::id().unwrap(), true);

    launch::remote_launch(slave_main, Some(&app_cfg), lcore_id).unwrap();

    ethapp::main(&mut app_cfg);

    app_cfg.exit_now = true;

    lcore::foreach_slave(|lcore_id| launch::wait_lcore(lcore_id));
}
