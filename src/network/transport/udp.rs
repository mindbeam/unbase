use std::net::UdpSocket;
use std::thread;
use std::str;
use std::mem;

use super::*;
use std::sync::mpsc;
use std::sync::{Arc,Mutex};
use slab::MemoOrigin;
use memo::*;
// use std::collections::BTreeMap;
use super::packet::*;

use serde::de::*;
use super::packet::serde::PacketSeed;

use serde_json;// {serialize as bin_serialize, deserialize as bin_deserialize};

#[derive(Clone)]
pub struct TransportUDP {
    shared: Arc<Mutex<TransportUDPInternal>>,
    // TEMPORARY - TODO: remove Arc<Mutex<>> here and instead make transmitters Send but not sync
    tx_channel: Arc<Mutex<mpsc::Sender<(TransportAddressUDP,Packet)>>>
}
struct TransportUDPInternal {
    socket: Arc<UdpSocket>,
    tx_thread: Option<thread::JoinHandle<()>>,
    rx_thread: Option<thread::JoinHandle<()>>,
    network: Option<WeakNetwork>,
    address: TransportAddressUDP
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TransportAddressUDP {
    address: String
}

impl TransportUDP {
    pub fn new (address: String) -> Self{
        let socket = Arc::new( UdpSocket::bind(address.clone()).expect("UdpSocket::bind") );

        let (tx_thread,tx_channel) = Self::setup_tx_thread(socket.clone());

        TransportUDP {
            tx_channel: Arc::new(Mutex::new(tx_channel)),
            shared: Arc::new(Mutex::new(
                TransportUDPInternal {
                    socket: socket,
                    rx_thread: None,
                    tx_thread: Some(tx_thread),
                    network: None,
                    address: TransportAddressUDP{ address : address }
                }
            ))
        }
    }

    fn setup_tx_thread (socket: Arc<UdpSocket>) -> (thread::JoinHandle<()>,mpsc::Sender<(TransportAddressUDP, Packet)>){
        let (tx_channel, rx_channel) = mpsc::channel::<(TransportAddressUDP,Packet)>();

        let tx_thread : thread::JoinHandle<()> = thread::spawn(move || {
            //let mut buf = [0; 65536];
            println!("Started TX Thread");
            loop {

                if let Ok((to_address, packet)) = rx_channel.recv() {
                    println!("GOT MEMO TO TRANSMIT");
                    let b = serde_json::to_vec(&packet).expect("serde_json::to_vec");

                    //HACK: we're trusting that each memo is smaller than 64k
                    socket.send_to(&b, &to_address.address).expect("Failed to send");
                    println!("SENT UDP PACKET ({}) {:?}", &to_address.address, String::from_utf8(b));
                }else{
                    break;
                }
            };
    });

        (tx_thread, tx_channel)
    }
    pub fn seed_address_from_string (&self, address_string: String) {

        let to_address = TransportAddressUDP{ address: address_string };

        let net;
        let my_address;
        {
            let shared = self.shared.lock().expect("TransportUDP.shared.lock");
            my_address = shared.address.clone();

            if let Some(ref n) = shared.network {
                net = n.upgrade().expect("Network upgrade");
            }else{
                panic!("Attempt to use uninitialized transport");
            }
        };

        for my_slab in net.get_all_local_slabs() {

            let presence = SlabPresence {
                slab_id: my_slab.id,
                transport_address: TransportAddress::UDP( my_address.clone() ),
                anticipated_lifetime: SlabAnticipatedLifetime::Unknown
            };

            let hello = Memo::new_basic_noparent(
                my_slab.gen_memo_id(),
                0,
                MemoBody::SlabPresence(presence)
            );

            self.send(
                &my_slab.get_ref(),
                0,
                hello,
                to_address.clone()
            );
        }

    }
    pub fn send (&self, from: &SlabRef, to_slab_id: SlabId, memo: Memo, address : TransportAddressUDP) {
        let packet = Packet{
            to_slab_id: to_slab_id,
            from_slab_id: from.slab_id,
            memo: memo
        };

        println!("TransportUDP.send({:?})", packet );
        // HACK HACK HACK lose the mutex here
        self.tx_channel.lock().unwrap().send( (address, packet) ).unwrap();
    }
}

impl Transport for TransportUDP {
    fn is_local (&self) -> bool {
        false
    }
    fn make_transmitter (&self, args: &TransmitterArgs ) -> Option<Transmitter> {

        if let &TransmitterArgs::Remote(slab_id,address) = args {
            if let &TransportAddress::UDP(ref udp_address) = address {

                let tx = TransmitterUDP{
                    slab_id: *slab_id,
                    address: udp_address.clone(),
                    tx_channel: self.tx_channel.clone(),
                };

                Some(Transmitter::new( Box::new(tx) ))
            }else{
                None
            }
        }else{
            None
        }
    }

    fn bind_network(&self, net: &Network) {
        let mut shared = self.shared.lock().unwrap();
        if let Some(_) = (*shared).rx_thread {
            panic!("already bound to network");
        }

        let rx_socket = shared.socket.clone();
        //let dispatcher = TransportUDPDispatcher::new(net.clone());

        let net_weak = net.weak();
        let rx_handle : thread::JoinHandle<()> = thread::spawn(move || {
            let mut buf = [0; 65536];

            loop {
                let (amt, src) = rx_socket.recv_from(&mut buf).unwrap();
                println!("GOT UDP PACKET");

                if let Some(net) = net_weak.upgrade() {
                    println!("GOT UDP PACKET - 1");

                    //TODO: create a protocol encode/decode module and abstract away the serde stuff
                    //ouch, my brain - I Think I finally understand ser::de::DeserializeSeed
                    let mut deserializer = serde_json::Deserializer::from_slice(&buf[0..amt]);
                    let packet : Packet;
                    {
                        let packet_seed : PacketSeed = PacketSeed{ net: &net };
                        packet = packet_seed.deserialize(&mut deserializer).unwrap();
                    }
                    println!("GOT UDP PACKET - 2");

                    // TODO: create packet.get_presence ?
                    // TODO: cache this
                    let from_presence =  SlabPresence{
                        slab_id: packet.from_slab_id,
                        transport_address: TransportAddress::UDP(TransportAddressUDP{ address: src.to_string() }),
                        anticipated_lifetime: SlabAnticipatedLifetime::Unknown
                    };
                    println!("GOT UDP PACKET - 3");

                    net.distribute_memos(&from_presence, packet);

                }
            };
        });

        shared.rx_thread = Some(rx_handle);
        shared.network = Some(net.weak());

    }
    fn get_return_address  ( &self, address: &TransportAddress ) -> Option<TransportAddress> {
        if let TransportAddress::UDP(_) = *address {
            let shared = self.shared.lock().unwrap();
            Some(TransportAddress::UDP(shared.address.clone()))
        }else{
            None
        }
    }
}

/*
impl Drop for TransportUDP{
    fn drop(&mut self) {
        let mut shared = self.shared.lock().unwrap();
        println!("# TransportUDP({:?}).drop", shared.address);
        let mut tx_thread = None;
        let mut rx_thread = None;
        mem::swap(&mut tx_thread,&mut shared.tx_thread);
        mem::swap(&mut rx_thread,&mut shared.rx_thread);
        tx_thread.unwrap().join().unwrap();
        rx_thread.unwrap().join().unwrap();
        // TODO: Drop all observers? Or perhaps observers should drop the slab (weak ref directionality)
    }
}
*/

pub struct TransmitterUDP{
    pub slab_id: SlabId,
    address: TransportAddressUDP,
    // HACK HACK HACK - lose the Arc<Mutex<>> here by making transmitter Send, but not Sync
    tx_channel: Arc<Mutex<mpsc::Sender<(TransportAddressUDP,Packet)>>>
}
impl DynamicDispatchTransmitter for TransmitterUDP {
    fn send (&self, from: &SlabRef, memo: Memo) {

        let packet = Packet {
            to_slab_id: self.slab_id,
            from_slab_id: from.slab_id,
            memo: memo
        };

        self.tx_channel.lock().unwrap().send((self.address.clone(), packet)).unwrap();
    }
}


/*
struct TransportUDPDispatcher{
    net: WeakNet,
    slabmap: HashMap<SlabId, Slab>,
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
*/
