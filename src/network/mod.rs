extern crate linked_hash_map;

pub mod transport;
pub mod packet;

pub use slab::{SlabRef, SlabPresence, SlabAnticipatedLifetime};
pub use self::transport::{Transport,TransportAddress,Transmitter,TransmitterArgs};
pub use self::packet::Packet;
use util::system_creator::SystemCreator;

use std::sync::{Arc, Weak, Mutex};
use std::fmt;
use slab::{Slab,WeakSlab,SlabId};
use memorefhead::MemoRefHead;

struct NetworkInternals {
    next_slab_id: u32,
    slabs:     Vec<WeakSlab>,
    transports: Vec<Box<Transport + Send + Sync>>,
    root_index_seed: Option<MemoRefHead>,
    create_new_system: bool
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
    /// Create a new network struct
    /// In production, this is the one you want
    pub fn new () -> Network {
        Self::new_inner(false)
    }
    /// In test cases, you want to create a wholly new unbase system.
    /// You should not be using this in production, except the *first* time ever for that system
    pub fn create_new_system () -> Network {
        Self::new_inner(true)
    }
    fn new_inner (create_new_system: bool) -> Network {

        let internals = NetworkInternals {
            next_slab_id: 0,
            slabs:     Vec::new(),
            transports: Vec::new(),
            root_index_seed: None,
            create_new_system: create_new_system
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
        println!("MARK A");
        let mut internals = self.shared.internals.lock().unwrap();
        println!("MARK B");
        internals.get_all_local_slabs()
    }
    pub fn get_slab (&self, slab_id: SlabId ) -> Option<Slab> {
        let mut internals = self.shared.internals.lock().unwrap();
        internals.get_slab(slab_id)
    }
    pub fn get_representative_slab (&self) -> Option<Slab>{
        let mut internals = self.shared.internals.lock().unwrap();
        internals.get_representative_slab()
    }
    pub fn get_transmitter (&self, args: TransmitterArgs ) -> Option<Transmitter> {

        let internals = self.shared.internals.lock().unwrap();
        for transport in internals.transports.iter() {
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
    pub fn register_local_slab(&self, new_slab: &Slab) {
        println!("# register_slab {:?}", new_slab );

        // Probably won't use transports in quite the same way in the future

        let mut internals = self.shared.internals.lock().unwrap();

        for prev_slab in internals.get_all_local_slabs() {
            prev_slab.slabref_from_local_slab( new_slab );
            new_slab.slabref_from_local_slab( &prev_slab );
        }

        internals.slabs.insert(0, new_slab.weak() );
    }

    pub fn get_root_index_seed(&self) -> Option<MemoRefHead> {

        let internals = self.shared.internals.lock().unwrap();

        match internals.root_index_seed {
            Some(ref s) => {
                Some(s.clone())
            }
            None => {
                None
            }
        }

    }
    pub fn conditionally_generate_root_index_seed (&self, slab: &Slab) -> bool {
        let mut internals = self.shared.internals.lock().unwrap();

        if let None = internals.root_index_seed {
            if internals.create_new_system {
                // I'm a new system, so I can do this!
                let seed = SystemCreator::generate_root_index_seed( slab );
                internals.root_index_seed = Some(seed.clone());
                return true;
            }
        }

        false
    }
    /// When we receive a root_index_seed from a peer slab that's already attached to a system,
    /// we need to apply it in order to "join" the same system
    ///
    /// TODO: how do we decide if we want to accept this?
    ///       do we just take any system seed that is sent to us when unseeded?
    ///       Probably good enough for Alpha, but obviously not good enough for Beta
    pub fn apply_root_index_seed(&self, _presence: &SlabPresence, root_index_seed: &MemoRefHead ) -> bool {
        let mut internals = self.shared.internals.lock().unwrap();

        match internals.root_index_seed {
            Some(_) => {
                // TODO: scrutinize the received root_index_seed to see if our existing seed descends it, or it descends ours
                //       if neither is the case ( apply currently allows this ) then reject the root_index_seed and return false
                //       this is use to determine if the SlabPresence should be blackholed or not

                // let did_apply : bool = internals.root_index_seed.apply_disallow_diverse_root(  root_index_seed )
                // did_apply

                true // be lenient for now. Not ok for Alpha
            }
            None => {
                internals.root_index_seed = Some(root_index_seed.clone());
                true
            }
        }
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
    fn get_representative_slab ( &mut self ) -> Option<Slab> {
        for weak in self.slabs.iter() {
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

            println!("# > Dropping NetworkInternals B");
        self.transports.clear();

            println!("# > Dropping NetworkInternals C");
        self.root_index_seed.take();

            println!("# > Dropping NetworkInternals D");

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
