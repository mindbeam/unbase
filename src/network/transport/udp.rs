use std::net::UdpSocket;
use std::thread;
use std::str;
use std::mem;

use super::*;
use std::sync::mpsc;
use std::sync::{Arc,Mutex};
use slab::*;
use memo::Memo;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct TransportUDP {
    shared: Arc<Mutex<TransportUDPInternal>>,
    tx_channel: Arc<mpsc::Sender<Memo>>
}
struct TransportUDPInternal {
    socket: Arc<UdpSocket>,
    tx_thread: Option<thread::JoinHandle<()>>,
    rx_thread: Option<thread::JoinHandle<()>>,
    address: String
}

impl TransportUDP {
    // TODO: Potentially, make this return an Arc of itself.
    pub fn new (address: String) -> Self{
        let socket = Arc::new( UdpSocket::bind(address.clone()).unwrap() );

        let (tx_thread,tx_channel) = Self::setup_tx_thread(socket.clone());

        TransportUDP {
            tx_channel: Arc::new(tx_channel),
            shared: Arc::new(Mutex::new(
                TransportUDPInternal {
                    socket: socket,
                    rx_thread: None,
                    tx_thread: Some(tx_thread),
                    address: address
                }
            ))
        }
    }

    fn setup_tx_thread (socket: Arc<UdpSocket>) -> (thread::JoinHandle<()>,mpsc::Sender<Memo>){
        let (tx_channel, rx_channel) = mpsc::channel::<Memo>();

        let tx_thread : thread::JoinHandle<()> = thread::spawn(move || {
            //let mut buf = [0; 65536];

            loop {
                let envelope = rx_channel.recv().unwrap();

                let dest = envelope.dest.clone();
                for packet in envelope.packet_iter() {
                    socket.send_to(packet, &dest)?;
                }
            };
        });

        (tx_thread, tx_channel)
    }
    pub fn seed_address (&self, address: String){

        self.send(hello, address);
    }
}

impl Transport for TransportUDP {
    fn is_local (&self) -> bool {
        false
    }
    fn make_transmitter (&self, args: TransmitterArgs ) -> Result<Transmitter,String> {
        if let TransmitterArgs::Remote(address) = args {
            let tx = TransmitterUDP{
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
        if let Some(_) = (*shared).rx_thread {
            panic!("already bound to network");
        }

        let rx_socket = shared.socket.clone();

        let dispatcher = TransportUDPDispatcher::new(net.clone());

        let rx_handle : thread::JoinHandle<()> = thread::spawn(move || {
            let mut buf = [0; 65536];

            loop {
                println!("INSIDE UDP THREAD");
                let (amt, src) = rx_socket.recv_from(&mut buf).unwrap();
                dispatcher.got_packet( &amt, &src );
            };
        });

        shared.rx_thread = Some(rx_handle);

    }

}

impl Drop for TransportUDP{
    fn drop(&mut self) {
        let mut shared = self.shared.lock().unwrap();
        println!("# TransportUDP({}).drop", shared.address);
        let mut tx_thread = None;
        let mut rx_thread = None;
        mem::swap(&mut tx_thread,&mut shared.tx_thread);
        mem::swap(&mut rx_thread,&mut shared.rx_thread);
        tx_thread.unwrap().join().unwrap();
        rx_thread.unwrap().join().unwrap();
        // TODO: Drop all observers? Or perhaps observers should drop the slab (weak ref directionality)
    }
}

pub struct TransmitterUDP{
    pub slab_id: SlabId,
    address: String,
    socket: Arc<UdpSocket>
}
impl DynamicDispatchTransmitter for TransmitterUDP {
    fn send (&self, from: &SlabRef, memo: Memo) {
        unimplemented!()
    }
}



struct TransportUDPDispatcher{
    net: WeakNet,
    slabmap: HashMap<SlabId, Slab>,
    active_envelopes: BTreeMap<(Source,EnvelopeId),Envelope>
}

impl TransportUDPDispatcher{
    pub fn got_packet(&mut self, buf: &str, source: &str) {
        if let packet = serde_json::from_str(buf) {

            let envelope = self.active_envelopes.entry((source,packet.envelope_id))
                .or_insert( Envelope::new(source, packet.envelope_id) );
            envelope.add_packet( packet );

            if ( envelope.is_dispatchable() ){
                let slab = match self.slabmap.entry( envelope.slab_id ) {
                    Entry::Occupied(e) => {
                        e.get().clone()
                    }
                    Entry::Vacant(e) => {
                        match net.upgrade().unwrap().get_slab( envelope.slab_id ) {
                            Some(slab) => {
                                e.set(slab)
                            }
                            None => {
                                self.active_envelopes.delete();
                                return;
                            }
                        }
                    }
                };

                slab.put_memos( MemoOrigin::Remote, envelope.yield_memos(), true );
                if envelope.is_completed() {
                    self.slabmap.remove()
                }
            }

        }

        // deserialize packet
        // assemble envelope
        // ask envelope for dest slabs
        // dispatch envelope memos to each dest slab we can find
        self.got_packet()


        if let Some(_net) = weak_net.upgrade() {
            println!("GOT DATAGRAM ({}, {}, {:?})", amt, src, str::from_utf8(&buf[0..amt]).unwrap() );

        }else{
            break;
        }
    }
}
