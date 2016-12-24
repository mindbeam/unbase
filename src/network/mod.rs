extern crate linked_hash_map;

pub mod peer;
use self::peer::*;

//use std::thread;
//use std::sync::mpsc::{Sender, channel};
//use std::thread::JoinHandle;
use std::sync::{Arc, Mutex, Weak};
use std::fmt;
use slab::{Slab, SlabOuter};
use memo::Memo;

struct NetworkInternals {
    next_slab_id: u32,
    slabs: Vec<Weak<Slab>>
}

pub struct NetworkShared {
    internals: Mutex<NetworkInternals>,
    //tx_thread: Option<JoinHandle<()>>,
}

#[derive(Clone)]
pub struct Network {
    shared: Arc<NetworkShared>
}

impl Network {
    pub fn new() -> Network {

        let internals = NetworkInternals {
            next_slab_id: 0,
            slabs: Vec::new(),
        };

        //let (tx_sender, tx_receiver) = channel();

        let shared = NetworkShared {
            internals: Mutex::new(internals),
            //tx_thread: None,
        };

        let net = Network {
            shared: Arc::new(shared),
            //tx_sender: tx_sender,
        };
/*
        let net2 = net.clone();
        let tx_thread = thread::spawn(move || {
            for tx_item in tx_receiver.iter() {
                net2.queue_memos(tx_item);
            }
        });

        shared.tx_thread = Some(tx_thread);
        */

        net
    }
    pub fn generate_slab_id(&self) -> u32 {
        let mut internals = self.shared.internals.lock().unwrap();
        internals.next_slab_id += 1;

        internals.next_slab_id
    }
    pub fn register_slab(&self, slab: &SlabOuter) {
        let mut internals = self.shared.internals.lock().unwrap();
        println!("register_slab {:?}", slab );
        // TODO: convert this into a iter generator that automatically expunges missing slabs.

        for prev_slab in internals.get_slabs() {
            slab.add_peer( SlabRef::new( &prev_slab ) );
            prev_slab.add_peer( SlabRef::new( slab ) );
        }

        internals.slabs.insert( 0, Arc::downgrade(slab) );

    }

/*    pub fn get_local_peers(&self) -> Vec<SlabRef>{
        let mut internals = self.shared.internals.lock().unwrap();

        let mut slabrefs : Vec<SlabRef> = Vec::new();

        for (_, slab_item) in internals.slab_map.iter_mut() {
            match slab_item.slab.upgrade() {
                Some(slab) => {
                    let slabref = SlabRef {
                        slab: slab
                    };
                    slabrefs.push(slabref);
                },
                None => {}
            }
        }

        slabrefs
    }
*/

    // TODO: fancy stuff like random delivery, losing memos, delivering to subsets of peers, etc
    // TODO: convert slab_map to a vec, and use references internally
    pub fn deliver_all_memos(&self) {
        let mut internals = self.shared.internals.lock().unwrap();

        for slab in internals.get_slabs().iter_mut() {
            slab.deliver_all_memos();
        }
    }
}

impl NetworkInternals {

    fn get_slabs (&mut self) -> Vec<SlabOuter> {
        let mut res: Vec<SlabOuter> = Vec::with_capacity(self.slabs.len());
        //let mut missing : Vec<usize> = Vec::new();

        for slab in self.slabs.iter_mut() {
            match slab.upgrade() {
                Some(s) => {
                    res.push( s );
                },
                None => {
                    // TODO: expunge freed slabs
                }
            }
        }

        res
    }
}

impl fmt::Debug for Network {
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
