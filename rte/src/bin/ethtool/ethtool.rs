use std::mem;
use std::result;
use std::sync::Mutex;

use rte::*;

pub const MAX_PORTS: u8 = RTE_MAX_ETHPORTS as u8;

pub const MAX_BURST_LENGTH: usize = 32;

pub struct TxQueuePort {
    pub cnt_unsent: usize,
    pub buf_frames: [mbuf::RawMbufPtr; MAX_BURST_LENGTH],
}

pub struct AppPort {
    pub mac_addr: ether::EtherAddr,
    pub txq: TxQueuePort,
    pub port_id: u8,
    pub port_active: bool,
    pub port_dirty: bool,
    pub pkt_pool: mempool::RawMemoryPoolPtr,
}

impl Default for AppPort {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

pub struct AppConfig {
    pub ports: Vec<Mutex<AppPort>>,
    pub exit_now: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

impl AppConfig {
    pub fn new(ports: u32) -> AppConfig {
        AppConfig {
            ports: (0..ports).map(|_| Mutex::new(AppPort::default())).collect(),
            exit_now: false,
        }
    }

    pub fn lock_port<T, F>(&self, port: u8, callback: F) -> result::Result<T, String>
        where F: Fn(&mut AppPort, &ethdev::EthDevice) -> result::Result<T, String>
    {
        match self.ports.iter().nth(port as usize) {
            Some(mutex) => {
                let dev = ethdev::EthDevice::from(port);

                if !dev.is_valid() {
                    Err(format!("port {} is invalid", port))
                } else {
                    match mutex.lock() {
                        Ok(mut guard) => {
                            let app_port = &mut *guard;

                            callback(app_port, &dev)
                        }
                        Err(err) => Err(format!("fail to lock port {}, {}", port, err)),
                    }
                }
            }
            _ => Err(format!("port number {} is invalid", port)),
        }
    }
}
