#[macro_use]
extern crate log;
extern crate env_logger;
extern crate libc;

#[macro_use]
extern crate rte;

mod ethtool;
mod ethapp;

use std::mem;
use std::env;
use std::sync::{Arc, Mutex};

use rte::*;

const MAX_PORTS: u8 = RTE_MAX_ETHPORTS as u8;

const MAX_BURST_LENGTH: usize = 32;
const PORT_RX_QUEUE_SIZE: u16 = 128;
const PORT_TX_QUEUE_SIZE: u16 = 256;
const PKTPOOL_EXTRA_SIZE: u16 = 512;
const PKTPOOL_CACHE: u32 = 32;

const EXIT_FAILURE: i32 = -1;

struct TxQueuePort {
    cnt_unsent: usize,
    buf_frames: [mbuf::RawMbufPtr; MAX_BURST_LENGTH],
}

struct AppPort {
    mac_addr: ether::EtherAddr,
    txq: TxQueuePort,
    lock: Option<Arc<Mutex<u32>>>,
    port_active: bool,
    port_dirty: bool,
    idx_port: u32,
    pkt_pool: Option<mempool::RawMemoryPool>,
}

struct AppConfig {
    ports: [AppPort; MAX_PORTS as usize],
    cnt_ports: u32,
    exit_now: bool,
}

impl AppConfig {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

fn setup_ports(app_cfg: &mut AppConfig) {
    let port_conf = ethdev::EthConf::default();

    for portid in 0..app_cfg.cnt_ports {
        let app_port = &mut app_cfg.ports[portid as usize];

        let dev = ethdev::EthDevice::from(portid as u8);
        let dev_info = dev.info();

        let info: &ethdev::RawEthDeviceInfo = &*dev_info;

        let size_pktpool = info.rx_desc_lim.nb_max + info.tx_desc_lim.nb_max + PKTPOOL_EXTRA_SIZE;

        app_port.pkt_pool = Some(mbuf::pktmbuf_pool_create(&format!("pkt_pool{}", portid),
                                                           size_pktpool as u32,
                                                           PKTPOOL_CACHE,
                                                           0,
                                                           mbuf::RTE_MBUF_DEFAULT_BUF_SIZE,
                                                           eal::socket_id())
            .expect("create mbuf pool failed"));

        println!("Init port {}..\n", portid);

        app_port.mac_addr = dev.macaddr();
        app_port.port_active = true;
        app_port.idx_port = portid;
        app_port.lock = Some(Arc::new(Mutex::new(0)));

        dev.configure(1, 1, &port_conf)
            .expect(format!("fail to configure device: port={}", portid).as_str());

        // init one RX queue
        dev.rx_queue_setup(0, PORT_RX_QUEUE_SIZE, None, &app_port.pkt_pool.unwrap())
            .expect(format!("fail to setup device rx queue: port={}", portid).as_str());

        // init one TX queue on each port
        dev.tx_queue_setup(0, PORT_TX_QUEUE_SIZE, None)
            .expect(format!("fail to setup device tx queue: port={}", portid).as_str());

        // Start device
        dev.start().expect(format!("fail to start device: port={}", portid).as_str());

        dev.promiscuous_enable();
    }
}

fn process_frame(app_port: &AppPort, frame: mbuf::RawMbufPtr) {
    let ether_hdr = unsafe { &mut *pktmbuf_mtod!(frame, *mut ether::EtherHdr) };

    ether::EtherAddr::copy(&ether_hdr.s_addr, &mut ether_hdr.d_addr);
    ether::EtherAddr::copy(&app_port.mac_addr, &mut ether_hdr.s_addr);
}

extern "C" fn slave_main(app_cfg: &mut AppConfig) -> i32 {
    while !app_cfg.exit_now {
        for portid in 0..app_cfg.cnt_ports {
            let app_port = &mut app_cfg.ports[portid as usize];

            // Check that port is active and unlocked
            if let Some(ref lock) = app_port.lock {
                if let Ok(_) = lock.try_lock() {
                    if !app_port.port_active {
                        continue;
                    }

                    let dev = ethdev::EthDevice::from(portid as u8);

                    // MAC address was updated
                    if app_port.port_dirty {
                        app_port.mac_addr = dev.macaddr();
                        app_port.port_dirty = false;
                    }

                    // Incoming frames
                    let cnt_recv_frames =
                        dev.rx_burst(0, &mut app_port.txq.buf_frames[app_port.txq.cnt_unsent..]);

                    if cnt_recv_frames > 0 {
                        for frame in
                            &app_port.txq.buf_frames[app_port.txq.cnt_unsent..app_port.txq
                            .cnt_unsent +
                                                                              cnt_recv_frames] {
                            process_frame(&app_port, *frame);
                        }

                        app_port.txq.cnt_unsent += cnt_recv_frames
                    }

                    // Outgoing frames
                    if app_port.txq.cnt_unsent > 0 {
                        let cnt_sent =
                            dev.tx_burst(0,
                                         &mut app_port.txq.buf_frames[..app_port.txq.cnt_unsent]);

                        for i in cnt_sent..app_port.txq.cnt_unsent {
                            app_port.txq.buf_frames[i - cnt_sent] = app_port.txq.buf_frames[i];
                        }
                    }
                }
            }
        }
    }

    0
}

fn main() {
    env_logger::init().unwrap();

    let args: Vec<String> = env::args().collect();

    // Init runtime enviornment
    eal::init(&args).expect("Cannot init EAL");

    let mut app_cfg = AppConfig::default();

    app_cfg.cnt_ports = match ethdev::count() {
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

    println!("Number of NICs: {}", app_cfg.cnt_ports);

    if lcore::count() < 2 {
        eal::exit(EXIT_FAILURE, "No available slave core!\n");
    }

    setup_ports(&mut app_cfg);

    // Assume there is an available slave..
    let lcore_id = lcore::next(lcore::id().unwrap(), true);

    launch::remote_launch(unsafe { mem::transmute(slave_main) },
                          Some(&app_cfg),
                          lcore_id)
        .unwrap();

    ethapp::main();

    app_cfg.exit_now = true;

    lcore::foreach_slave(|lcore_id| launch::wait_lcore(lcore_id));
}
