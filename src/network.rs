extern crate linked_hash_map;


use std::sync::{Arc,Mutex};
use linked_hash_map::LinkedHashMap;
use std::sync::mpsc::{Sender,Receiver,channel};
use std::{fmt};
use slab::Slab;
use memo::Memo;


pub struct PeerSlab {
    id: u32
}
pub enum PeerSpec {
    Any (u8),
    List(Vec<PeerSlab>)
}

impl fmt::Debug for PeerSlab{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("PeerSlab")
           .field("id", &self.id)
           .finish()
    }
}

impl fmt::Debug for PeerSpec {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {

        let mut dbg = fmt.debug_struct("PeerSpec");

        match self {
            &PeerSpec::Any(c)  => {
                dbg.field("Any", &c);
            },
            &PeerSpec::List(ref v) => {
                for p in v {
                    dbg.field("Peer", &p);
                }
            }
        };

        dbg.finish()
    }
}

struct NetworkInternals{
    next_slab_id: u32,
    tx_map:   LinkedHashMap<u32, Sender<Memo>>
}
pub struct NetworkShared {
    internals: Mutex<NetworkInternals>
}

#[derive(Clone)]
pub struct Network {
    shared: Arc<NetworkShared>
}

/// Returns a new `Network` referencing the same internal shared object as `self`.
/*
impl Clone for Network {
    fn clone(&self) -> Network {
        Network {
            shared: self.shared.clone()
        }
    }
}
*/

impl fmt::Debug for Network{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let inner = self.shared.internals.lock().unwrap();

        fmt.debug_struct("Network")
           .field("next_slab_id", &inner.next_slab_id)
           .finish()
    }
}

impl Network {
    pub fn new() -> Network {

        let internals = NetworkInternals {
            next_slab_id: 0,
            tx_map: LinkedHashMap::new()
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
        println!("{:?} - {:?}", memo, peer_spec);
        //unimplemented!()
    }
}
