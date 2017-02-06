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
use slab::{Slab,WeakSlab};

struct NetworkInternals {
    next_slab_id: u32,
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

        for prev_slab_ref in internals.get_slabrefs() {
            slab.add_peer( prev_slab_ref.clone() );
            // TODO: Resolve the slab vs slabref issue - should this be a memo?
            prev_slab_ref.add_peer( slab_ref.clone() );
        }

        internals.slab_refs.insert( 0, slab_ref );

    }
}

impl NetworkInternals {

    fn get_slabrefs (&mut self) -> Vec<SlabRef> {

        // TODO: convert this into a iter generator that automatically expunges missing slabs.
        let mut res: Vec<SlabRef> = Vec::with_capacity(self.slab_refs.len());
        //let mut missing : Vec<usize> = Vec::new();

        for slabref in self.slab_refs.iter_mut() {
            // TODO who figures out if a slab is a good peer or not?
            //if slabref.is_resident() {
                res.push( slabref.clone() );
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
