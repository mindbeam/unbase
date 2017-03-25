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
    fn make_transmitter (&self, args: &TransmitterArgs ) -> Option<Transmitter> {
        if let &TransmitterArgs::Local(slab) = args {
            let my_slab = slab.clone();
            let (tx_channel, rx_channel) = mpsc::channel::<(SlabId,Memo)>();

            let tx_thread : thread::JoinHandle<()> = thread::spawn(move || {
                //let mut buf = [0; 65536];
                println!("Started TX Thread");
                loop {

                    if let Ok((from_slab, memo)) = rx_channel.recv() {
                        println!("CHANNEL RCV {:?}", memo);
                        my_slab.put_memos( &MemoOrigin::Local, vec![memo], true );
                    }else{
                        break;
                    }
                }
            });

            // TODO: Remove the mutex here. Consider moving transmitter out of slabref.
            //       Instead, have relevant parties request a transmitter clone from the network
            self.shared.lock().unwrap().tx_threads.push(tx_thread);
            Some(Transmitter::new_local(Mutex::new(tx_channel)))
        }else{
            None
        }

    }

    fn bind_network(&self, _net: &Network) {}
    fn unbind_network(&self, _net: &Network) {}

    fn get_return_address  ( &self, address: &TransportAddress ) -> Option<TransportAddress> {
        if let TransportAddress::Local = *address {
            Some(TransportAddress::Local)
        }else{
            None
        }
    }
}

impl Drop for LocalDirect {
    fn drop (&mut self) {
        let mut shared = self.shared.lock().unwrap();
        for thread in shared.tx_threads.drain(..) {
            thread.join().unwrap();
        }
    }
}
