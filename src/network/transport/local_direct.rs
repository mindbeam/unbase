use std::sync::{Arc,Mutex};
use std::thread;
use std::sync::mpsc;
use super::*;
use slab::MemoOrigin;
use memo::*;

#[derive(Clone)]
pub struct LocalDirect {
    shared: Arc<Mutex<Internal>>,
}
struct Internal {
    tx_threads: Vec<thread::JoinHandle<()>>
}

impl LocalDirect {
    // TODO: Potentially, make this return an Arc of itself.
    pub fn new () -> Self{
        LocalDirect {
            shared: Arc::new(Mutex::new(
                Internal {
                    tx_threads: Vec::new()
                }
            ))
        }
    }
}

impl Transport for LocalDirect {
    fn is_local (&self) -> bool {
        true
    }
    fn make_transmitter (&self, args: TransmitterArgs ) -> Option<Transmitter> {
        if let TransmitterArgs::Local(slab) = args {
            let (tx_channel, rx_channel) = mpsc::channel::<(SlabId,Memo)>();

            let tx_thread : thread::JoinHandle<()> = thread::spawn(move || {
                //let mut buf = [0; 65536];
                println!("Started TX Thread");
                loop {

                    if let Ok((from_slab, memo)) = rx_channel.recv() {
                        slab.put_memos( MemoOrigin::Local, vec![memo], true );
                    }else{
                        break;
                    }
                }
            });

            //self.shared.lock().unwrap().tx_threads
            Some(Transmitter::new_local(tx_channel))
        }else{
            None
        }

    }

    fn bind_network(&self, _net: &Network) {
        //nothing to see here folks
    }

    fn get_return_address  ( &self, address: &TransportAddress ) -> Option<TransportAddress> {
        if let TransportAddress::Local = *address {
            Some(TransportAddress::Local)
        }else{
            None
        }
    }
}
