extern crate linked_hash_map;

pub mod peer;
use self::peer::*;

use std::sync::{Arc,Mutex};
use linked_hash_map::LinkedHashMap;
use std::sync::mpsc::{Sender,Receiver,channel};
use std::{fmt};
use slab::Slab;
use memo::Memo;

struct TxItem {
    memo:     Memo,
    peer_spec: PeerSpec
}
struct NetworkInternals{
    next_slab_id: u32,
    tx_map:   LinkedHashMap<u32, Sender<Memo>>,
    tx_hopper: Vec<TxItem>
}
pub struct NetworkShared {
    internals: Mutex<NetworkInternals>
}

#[derive(Clone)]
pub struct Network {
    shared: Arc<NetworkShared>
}

impl Network {
    pub fn new() -> Network {

        let internals = NetworkInternals {
            next_slab_id: 0,
            tx_map: LinkedHashMap::new(),
            tx_hopper: Vec::new()
        };
        let shared = NetworkShared {
            internals: Mutex::new(internals)
        };
        Network {
            shared: Arc::new(shared)
        }
    }
    pub fn generate_slab_id(&self) -> u32 {
        let mut internals = self.shared.internals.lock().unwrap();
        internals.next_slab_id += 1;

        internals.next_slab_id
    }
    pub fn register_slab( &self, slab: &Slab ) -> (Receiver<Memo>) {
        let mut internals = self.shared.internals.lock().unwrap();

        let ( tx, rx  ) = channel();
        internals.tx_map.insert(slab.id,tx);
        rx
    }
    pub fn transmit_memo( &self, memo: Memo, peer_spec: PeerSpec) -> () {
        //println!("transmit_memo {:?} - {:?}", memo, peer_spec);

        let mut internals = self.shared.internals.lock().unwrap();
        internals.tx_hopper.push( TxItem {
            memo: memo,
            peer_spec: peer_spec
        });
    }

    pub fn deliver_memos (&self, mut number_of_memos: usize){
        // TODO: fancy stuff like random delivery, losing memos, delivering to subsets of peers, etc
        let mut internals = self.shared.internals.lock().unwrap();

        if number_of_memos == 0 {
            number_of_memos = internals.tx_hopper.len();
        }

        for _ in 0..number_of_memos {
            let tx_item = internals.tx_hopper.remove(0);

            match tx_item.peer_spec {
                self::peer::PeerSpec::Any(n) => {
                    let mut value_iter = internals.tx_map.values();

                    for _ in 1..n {
                        // naive for now
                        let tx_opt = value_iter.next();
                        match tx_opt {
                            Some(tx) => { tx.send( tx_item.memo.clone() ).unwrap() ; }
                            None =>  { }
                        };
                    }
                },
                _ => {}
            }
        }
    }
}


impl fmt::Debug for Network{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let inner = self.shared.internals.lock().unwrap();

        fmt.debug_struct("Network")
           .field("next_slab_id", &inner.next_slab_id)
           .finish()
    }
}
