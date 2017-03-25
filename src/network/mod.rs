extern crate linked_hash_map;

pub mod transport;
pub mod slabref;
pub mod packet;

pub use self::slabref::{SlabRef, SlabPresence, SlabAnticipatedLifetime};
pub use self::transport::{Transport};
pub use self::packet::Packet;
use self::transport::*;

use std::sync::{Arc, Weak, Mutex};
use std::fmt;
use slab::{Slab,WeakSlab,SlabId,MemoOrigin};
use memorefhead::MemoRefHead;

struct NetworkInternals {
    next_slab_id: u32,
    slabs:     Vec<WeakSlab>,
    slab_refs: Vec<SlabRef>,
    transports: Vec<Box<Transport + Send + Sync>>,
    root_index_seed: Option<MemoRefHead>
}

struct NetworkShared {
    internals: Mutex<NetworkInternals>
}

#[derive(Clone)]
pub struct Network {
    shared: Arc<NetworkShared>
}
pub struct WeakNetwork {
    shared: Weak<NetworkShared>
}

impl Network {
    pub fn new () -> Network {

        let internals = NetworkInternals {
            next_slab_id: 0,
            slabs:     Vec::new(),
            slab_refs: Vec::new(),
            transports: Vec::new(),
            root_index_seed: None
        };

        let shared = NetworkShared {
            internals: Mutex::new(internals)
        };

        let net = Network {
            shared: Arc::new(shared)
        };

        let localdirect = self::transport::LocalDirect::new();
        net.add_transport(Box::new(localdirect));

        net
    }
    pub fn hack_set_next_slab_id(&self, id: SlabId ){
        let mut internals = self.shared.internals.lock().unwrap();
        internals.next_slab_id = id;
    }
    pub fn weak (&self) -> WeakNetwork {
        WeakNetwork {
            shared: Arc::downgrade(&self.shared)
        }
    }
    pub fn add_transport (&self, transport: Box<Transport + Send + Sync> ) {
        let mut internals = self.shared.internals.lock().unwrap();

        if transport.is_local() {
            // Can only have one is_local transport at a time
            if let Some(removed) = internals.transports.iter_mut().position(|t| t.is_local())
                .map(|e| internals.transports.remove(e)) {
                    println!("Unbinding local transport");
                    removed.unbind_network(self);
            }
        }

        transport.bind_network(self);
        internals.transports.push(transport);
    }
    pub fn generate_slab_id(&self) -> u32 {
        let mut internals = self.shared.internals.lock().unwrap();

        let id = internals.next_slab_id;

        internals.next_slab_id += 1;

        id
    }
    pub fn get_all_local_slabs(&self) -> Vec<Slab> {
        let mut internals = self.shared.internals.lock().unwrap();
        internals.get_all_local_slabs()
    }
    pub fn get_slab (&mut self, slab_id: SlabId ) -> Option<Slab> {
        let mut internals = self.shared.internals.lock().unwrap();
        internals.get_slab(slab_id)
    }
    pub fn get_slabref (&mut self, slab_id: SlabId ) -> Option<&SlabRef> {
        let mut internals = self.shared.internals.lock().unwrap();
        internals.slab_refs.iter().find(|x| x.slab_id == slab_id )
    }
    pub fn distribute_memos(&self, from_presence: &SlabPresence, packet: Packet ) {
        println!("Network.distribute_memos");
        // TODO: optimize this. redundant mutex locking inside, weak slab upgrades, etc

        let from = self.assert_slabref_from_presence(from_presence);
        let memoorigin = MemoOrigin::Other(&from);

        let mut send_slabs = Vec::new();
        {
            let internals = self.shared.internals.lock().unwrap();

            for weak_slab in internals.slabs.iter(){
                if packet.to_slab_id == 0 || weak_slab.id == packet.to_slab_id {
                    if let Some(slab) = weak_slab.upgrade() {
                        send_slabs.push(slab);
                    }
                }
            }
        }
        // can't have the lock open any time we're putting memos
        // because some internal logic needs to access the network struct
        for slab in send_slabs {
            slab.put_memos( &memoorigin,vec![packet.memo.clone()], true);
        }
    }
    pub fn assert_slabref_from_presence(&self, presence: &SlabPresence) -> SlabRef {

        {
            let internals = self.shared.internals.lock().unwrap();
            match internals.slab_refs.iter().find(|r| r.presence == *presence ) {
                Some(slabref) => {
                    //TODO: should we update the slabref if the address is different?
                    //      or should we find/make a new slabref because its different?
                    return slabref.clone();
                }
                _ =>{}
            }
        }

        let slabref = SlabRef::new_from_presence(&presence, &self);
        self.shared.internals.lock().unwrap().slab_refs.push(slabref.clone());
        return slabref;
    }
    pub fn get_transmitter (&self, args: TransmitterArgs ) -> Option<Transmitter> {

        let internals = self.shared.internals.lock().unwrap();
        for transport in internals.transports.iter() {
            println!("Considering transport" );
            if let Some(transmitter) = transport.make_transmitter( &args ) {
                return Some(transmitter);
            }
        }
        None

    }
    pub fn get_return_address<'a>( &self, address: &TransportAddress ) -> Option<TransportAddress> {
        // We're just going to assume that we have an in-process transmitter, or freak out
        // Should probably do this more intelligently

        let internals = self.shared.internals.lock().unwrap();
        for transport in internals.transports.iter() {

            if let Some(return_address) = transport.get_return_address( address ) {
                return Some(return_address);
            }

        }
        None
    }
    pub fn register_slab(&self, slab: &Slab) -> SlabRef {
        println!("# register_slab {:?}", slab );

        // Probably won't use transports in quite the same way in the future

        let slab_ref = SlabRef::new_from_slab( &slab, &self );

        let mut internals = self.shared.internals.lock().unwrap();

        for prev_slab in internals.get_all_local_slabs() {
            prev_slab.inject_peer_slabref( slab_ref.clone() );
        }
        for prev_slab_ref in internals.get_slab_refs() {
            slab.inject_peer_slabref( prev_slab_ref.clone() );
        }

        internals.slab_refs.insert( 0, slab_ref.clone() );
        internals.slabs.insert(0, slab.weak() );

        slab_ref
    }

    pub fn get_root_index_seed(&self, slab: &Slab) -> MemoRefHead {

        let mut internals = self.shared.internals.lock().unwrap();

        match internals.root_index_seed {
            Some(ref s) => {
                return s.clone()
            }
            None => {}
        }

        let seed = slab.generate_root_index_seed();
        internals.root_index_seed = Some(seed.clone());
        seed
    }
}

impl NetworkInternals {

    fn get_slab (&mut self, slab_id: SlabId ) -> Option<Slab> {
        if let Some(weak) = self.slabs.iter().find(|s| s.id == slab_id ) {
            if let Some(slab) = weak.upgrade() {
                return Some(slab);
            }
        }

        return None;
    }
    fn get_all_local_slabs (&mut self) -> Vec<Slab> {
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
        println!("# > Dropping NetworkInternals");
    }
}

impl WeakNetwork {
    pub fn upgrade (&self) -> Option<Network> {
        match self.shared.upgrade() {
            Some(s) => Some( Network { shared: s } ),
            None    => None
        }
    }
}
