extern crate linked_hash_map;

pub mod channel;
pub mod slabref;
use self::slabref::*;
use self::channel::*;

//use std::thread;
//use std::sync::mpsc::{Sender, channel};
//use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};
use std::fmt;
use slab::{Slab,WeakSlab,SlabId};

struct NetworkInternals {
    next_slab_id: u32,
    slabs:     Vec<WeakSlab>,
    slab_refs: Vec<SlabRef>
}

pub struct NetworkShared {
    internals: Mutex<NetworkInternals>
    //send_sync_handle: Arc<Mutex<()>>
    //tx_thread: Option<JoinHandle<()>>,
}

#[derive(Clone)]
pub struct Network {
    shared: Arc<NetworkShared>,
    oculus_dei: OculusDei
}

pub struct NetworkAddr ();

impl Network {
    pub fn new( oculus_dei: &OculusDei ) -> Network {

        let internals = NetworkInternals {
            next_slab_id: 0,
            slabs:     Vec::new(),
            slab_refs: Vec::new()
        };

        let shared = NetworkShared {
            internals: Mutex::new(internals)
        };

        let net = Network {
            oculus_dei: oculus_dei.clone(),
            shared: Arc::new(shared)
        };

        net
    }
    pub fn generate_slab_id(&self) -> u32 {
        let mut internals = self.shared.internals.lock().unwrap();
        internals.next_slab_id += 1;

        internals.next_slab_id
    }
    pub fn get_slabref(&self, slab_id: SlabId) -> Option<SlabRef> {
        unimplemented!();
    }
    pub fn register_slab(&self, slab: &Slab) {
        println!("register_slab {:?}", slab );

        let sender = Sender{
                        source_point: XYZPoint{ x: 1000, y: 1000, z: 1000 },
                        dest_point:   XYZPoint{ x: 1000, y: 1000, z: 1000 },
                        oculus_dei:   self.oculus_dei.clone(),
                        dest:    slab.weak()
                    };

        let slab_ref = SlabRef::new( slab.id, sender );

        let mut internals = self.shared.internals.lock().unwrap();

        for prev_slab in internals.get_slabs() {
            prev_slab.inject_peer_slabref( slab_ref.clone() );
        }
        for prev_slab_ref in internals.get_slab_refs() {
            slab.inject_peer_slabref( prev_slab_ref.clone() );
        }

        internals.slab_refs.insert( 0, slab_ref );
        internals.slabs.insert(0, slab.weak() );

    }
}

impl NetworkInternals {


    fn get_slabs (&mut self) -> Vec<Slab> {
        // TODO: convert this into a iter generator that automatically expunges missing slabs.
        let mut res: Vec<Slab> = Vec::with_capacity(self.slabs.len());
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

    fn get_slab_refs (&mut self) -> Vec<SlabRef> {
        // TODO: convert this into a iter generator that automatically expunges missing slabs.
        let mut res: Vec<SlabRef> = Vec::with_capacity(self.slabs.len());
        //let mut missing : Vec<usize> = Vec::new();

        for slab_ref in self.slab_refs.iter() {
            //if slab_ref.is_resident() {
                res.push(slab_ref.clone());
            //}
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

impl Drop for NetworkInternals {
    fn drop(&mut self) {
        println!("> Dropping NetworkInternals");
    }
}
