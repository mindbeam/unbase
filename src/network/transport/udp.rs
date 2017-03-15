use std::net::UdpSocket;
use std::{thread, time};

use std::fmt;
use super::*;
use std::sync::{Arc,Mutex};
use slab::*;
use memo::Memo;

#[derive(Clone)]
pub struct Transport_UDP {
    shared: Arc<Mutex<Transport_UDP_Internal>>,
}
struct Transport_UDP_Internal {
    socket: Arc<UdpSocket>,
    rcv_thread: Option<thread::JoinHandle<Result<(),u32>>>
}

impl Transport_UDP {
    // TODO: Potentially, make this return an Arc of itself.
    pub fn new (address: String) -> Self{
        let mut socket = UdpSocket::bind(address).unwrap();

        Transport_UDP {
            shared: Arc::new(Mutex::new(
                Transport_UDP_Internal {
                    socket: Arc::new(socket),
                    rcv_thread: None
                }
            ))
        }
    }

}

impl Transport for Transport_UDP {
    fn is_local (&self) -> bool {
        false
    }
    fn make_transmitter (&self, args: TransmitterArgs ) -> Result<Transmitter,String> {
        if let TransmitterArgs::Remote(address) = args {
            let tx = Transmitter_UDP{
                slab_id: 0,
                address: address.to_owned(),
                socket: self.shared.lock().unwrap().socket.clone(),
            };

            Ok(Transmitter::new(Box::new(tx)))
        }else{
            Err("This transport is incapable of handling remote addresses".to_string())
        }

    }

    fn bind_network(&self, net: &Network) {
        let mut shared = self.shared.lock().unwrap();
        if let Some(_) = (*shared).rcv_thread {
            panic!("already bound to network");
        }

        let socket = shared.socket.clone();
        let weak_net = net.weak();
        shared.rcv_thread = thread::spawn(move || {
            loop {
                println!("INSIDE UDP THREAD");
                let mut buf = [0; 10];
                let (amt, src) = try!(socket.recv_from(&mut buf));
                if let Some(net) = weak_net.upgrade() {
                    println!("GOT DATAGRAM ({}, {}, {:?})", amt, src, buf );
                }else{
                    break;
                }
            };
        });

    }
}

pub struct Transmitter_UDP{
    pub slab_id: SlabId,
    address: String,
    socket: Arc<UdpSocket>
}
impl DynamicDispatchTransmitter for Transmitter_UDP {
    fn send (&self, from: &SlabRef, memo: Memo) {
        unimplemented!()
    }
}
