extern crate linked_hash_map;

pub mod peer;
use self::peer::*;

use std::sync::{Arc,Mutex,Weak};
use linked_hash_map::LinkedHashMap;
use std::{fmt};
use slab::{Slab,SlabOuter};
use memo::Memo;

pub struct TxItem {
    pub memo:     Memo,
    pub peer_spec: PeerSpec
}

struct SlabMapItem{
    slab:     Weak<Slab>,
    tx_queue: Vec<Memo>
}

struct NetworkInternals{
    next_slab_id: u32,
    slab_map:   LinkedHashMap<u32, SlabMapItem>
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
            slab_map: LinkedHashMap::new()
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
    pub fn register_slab( &self, slab: &SlabOuter ) {
        let mut internals = self.shared.internals.lock().unwrap();

        //TODO - employ weak refs to avoid memory leaks
        //       Uncertain which should be weak: net->slab or slab->net

        let item = SlabMapItem {
            slab:     Arc::downgrade(slab),
            tx_queue: Vec::new()
        };

        internals.slab_map.insert(slab.id, item);
    }
    pub fn transmit_memos ( &self, tx_items: Vec<TxItem>) {
        //println!("transmit_memo {:?} - {:?}", memo, peer_spec);

        let mut internals = self.shared.internals.lock().unwrap();

        for tx_item in tx_items {
            self.queue_memo(&mut internals, tx_item.memo, tx_item.peer_spec);
        }

        // TODO - configurably auto-deliver these memos
        //        punting for now, because we want the test suite to monkey with delivery
    }

    fn queue_memo (&self, internals: &mut NetworkInternals, memo: Memo, peer_spec: PeerSpec){
        use self::peer::PeerSpec::*;

        match peer_spec {
            Any(n) => {
                for (_,slab_item) in internals.slab_map.iter_mut().take(n as usize) {
                    slab_item.tx_queue.push( memo.clone() )
                }
            },
            List(slab_refs) => {
                for slab_ref in slab_refs {
                    match internals.slab_map.get_mut(&slab_ref.id) {
                        Some(slab_item) => {
                            slab_item.tx_queue.push(memo.clone())
                        },
                        None => {}
                    }
                }
            }
        }

    }

    // TODO: fancy stuff like random delivery, losing memos, delivering to subsets of peers, etc
    pub fn deliver_all_memos (&self){
        let mut internals = self.shared.internals.lock().unwrap();
        let slab_map = &mut internals.slab_map;

        let mut missing: Vec<u32> = Vec::new();

        for (k,slab_item) in slab_map.iter_mut() {
            let tx_queue = &mut slab_item.tx_queue;

            if tx_queue.len() > 0 {
                match slab_item.slab.upgrade() {
                    Some(slab) => {
                        slab.put_memos(tx_queue.drain(..).collect());
                    },
                    None => {
                        missing.push(*k)
                    }
                }
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

impl Drop for Network {
    fn drop(&mut self) {
        println!("> Dropping Network");
    }
}
