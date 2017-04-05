pub mod memo;
pub mod slabref;
pub mod memoref;
mod memohandling;

pub use self::slabref::{SlabRef,SlabRefInner,SlabPresence,SlabAnticipatedLifetime};
pub use self::memoref::{MemoRef,MemoRefShared,MemoRefPtr};
pub use self::memo::{Memo,MemoPeer,MemoPeerList};

use self::memo::*;
use network::transport::{TransportAddress,TransmitterArgs};

use std::fmt;
use std::sync::mpsc;
use std::sync::mpsc::channel;
use std::sync::{Arc,Mutex,Weak,MutexGuard};
use std::sync::atomic;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use network::{Network,Transmitter};
use subject::SubjectId;
use memorefhead::*;
use context::{Context,WeakContext};


/* Initial plan:
 * Initially use Mutex-managed internal struct to manage slab storage
 * TODO: refactor to use a lock-free hashmap or similar
 */

pub type SlabId = u32;

#[derive(Clone)]
pub struct Slab {
    pub id: SlabId,
    inner: Arc<SlabInner>
}

pub struct SlabShared{
    pub id: SlabId,
    memorefs_by_id: HashMap<MemoId,MemoRef>,
    memo_wait_channels: HashMap<MemoId,Vec<mpsc::Sender<Memo>>>,
    subject_subscriptions: HashMap<SubjectId, Vec<WeakContext>>,

    counters: SlabCounters,

    pub my_ref: SlabRef,
    peer_refs: Vec<SlabRef>,
    net: Network
}
struct SlabCounters{
    last_memo_id: u32,
    last_subject_id: u32,
    memos_received: u64,
    memos_redundantly_received: u64,
}
struct SlabInner {
    pub id: SlabId,
    shared: Mutex<SlabShared>
}

#[derive(Clone)]
pub struct WeakSlab{
    pub id: u32,
    inner: Weak<SlabInner>
}

//TODO: update OtherSlab to use MemoPeer?
#[derive(Debug)]
pub enum MemoOrigin<'a>{
    SameSlab,
    OtherSlab(&'a SlabRef, MemoPeeringStatus)
    // TODO: consider bifurcation into OtherSlabTrusted, OtherSlabUntrusted
    //       in cases where we want to reduce computational complexity by foregoing verification
}

impl Slab {
    pub fn new(net: &Network) -> Slab {
        let slab_id = net.generate_slab_id();

        let my_ref_inner = SlabRefInner {
            slab_id: slab_id,
            owning_slab_id: slab_id, // I own my own ref to me, obviously
            presence: Mutex::new(vec![]), // this bit is just for show
            tx: Mutex::new(Transmitter::new_blackhole()),
            return_address: TransportAddress::Local,
        };

        let my_ref = SlabRef(Arc::new(my_ref_inner));

        let shared = SlabShared {
            id: slab_id,
            memorefs_by_id:        HashMap::new(),
            memo_wait_channels:    HashMap::new(),
            subject_subscriptions: HashMap::new(),

            counters: SlabCounters {
                last_memo_id: 5000,
                last_subject_id: 0,
                memos_received: 0,
                memos_redundantly_received: 0,
            },

            my_ref: my_ref,
            peer_refs: Vec::new(),
            net: net.clone()
        };

        let me = Slab {
            id: slab_id,
            inner: Arc::new(SlabInner {
                id: slab_id,
                shared: Mutex::new(shared)
            })
        };

        net.register_local_slab(&me);

        net.conditionally_generate_root_index_seed(&me);

        me
    }

    pub fn inner (&self) -> MutexGuard<SlabShared> {
        self.inner.shared.lock().unwrap()
    }
    pub fn weak (&self) -> WeakSlab {
        WeakSlab {
            id: self.id,
            inner: Arc::downgrade(&self.inner)
        }
    }
    pub fn get_root_index_seed (&self) -> Option<MemoRefHead> {
    println!("get_root_index_seed A" );
        let net;
        {
            let shared = self.inner.shared.lock().unwrap();
            println!("get_root_index_seed B" );
            net = shared.net.clone();
        }

        println!("get_root_index_seed C" );
        let seed = net.get_root_index_seed();

        println!("get_root_index_seed D" );

        seed
    }
    // Convenience function for now, but may make sense to optimize this later
    pub fn put_memo (&self, memo_origin: &MemoOrigin, memo : Memo ) -> MemoRef {
        let mut memorefs = self.put_memos(memo_origin, vec![memo] );
        memorefs.pop().unwrap()
    }
    pub fn put_memos(&self, memo_origin: &MemoOrigin, memos : Vec<Memo> ) -> Vec<MemoRef> {
        if memos.len() == 0 { return Vec::new() }

        let mut shared = self.inner.shared.lock().unwrap();

        let memorefs = shared.put_memos(memo_origin, memos);

        memorefs

    }
    pub fn count_of_memorefs_resident( &self ) -> u32 {
        let shared = self.inner.shared.lock().unwrap();
        shared.memorefs_by_id.len() as u32
    }
    pub fn count_of_memos_received( &self ) -> u64 {
        let shared = self.inner.shared.lock().unwrap();
        shared.memos_received
    }
    pub fn count_of_memos_reduntantly_received( &self ) -> u64 {
        let shared = self.inner.shared.lock().unwrap();
        shared.memos_redundantly_received
    }

    pub fn peer_slab_count (&self) -> usize {
        let shared = self.inner.shared.lock().unwrap();

        println!("Slab({}).peer_slab_count = {}", self.id, shared.peer_refs.len() );
        shared.peer_refs.len()
    }
    pub fn create_context (&self) -> Context {
        Context::new(self)
    }
    pub fn subscribe_subject (&self, subject_id: u64, context: &Context) {
        let weakcontext : WeakContext = context.weak();

        let mut shared = self.inner.shared.lock().unwrap();

        if let Some(subs) = shared.subject_subscriptions.get_mut(&subject_id) {
            subs.push(weakcontext);
            return;
        }

        // Stoopid borrow checker
        shared.subject_subscriptions.insert(subject_id, vec![weakcontext]);
        return;
    }
    pub fn unsubscribe_subject (&self,  subject_id: u64, context: &Context ){
        println!("# Slab({}).unsubscribe_subject({})", self.id, subject_id);

        let mut shared = self.inner.shared.lock().unwrap();

        if let Some(subs) = shared.subject_subscriptions.get_mut(&subject_id) {
            let weak_context = context.weak();
            subs.retain(|c| {
                c.cmp(&weak_context)
            });
            return;
        }
    }
    pub fn memo_wait_channel (&self, memo_id: MemoId ) -> mpsc::Receiver<Memo> {

        let (tx, rx) = channel::<Memo>();
        let mut shared = self.inner.shared.lock().unwrap();

        match shared.memo_wait_channels.entry(memo_id) {
            Entry::Vacant(o)       => { o.insert( vec![tx] ); }
            Entry::Occupied(mut o) => { o.get_mut().push(tx); }
        };

        rx
    }
    pub fn remotize_memo_ids( &self, memo_ids: &[MemoId] ) {
        println!("# Slab({}).remotize_memo_ids({:?})", self.id, memo_ids);

        let mut memorefs : Vec<MemoRef> = Vec::with_capacity(memo_ids.len());

        {
            let shared = self.inner.shared.lock().unwrap();
            for memo_id in memo_ids.iter() {
                if let Some(memoref) = shared.memorefs_by_id.get(memo_id) {
                    memorefs.push( memoref.clone() )
                }
            }
        }

        for memoref in memorefs {
            memoref.remotize(&self)
        }
    }
}

impl WeakSlab {
    pub fn upgrade (&self) -> Option<Slab> {
        match self.inner.upgrade() {
            Some(i) => Some( Slab { id: self.id, inner: i } ),
            None    => None
        }
    }
}

impl SlabShared {
    pub fn generate_subject_id(&self) -> SubjectId {
        self.counters.last_subject_id += 1;
        (self.id as u64).rotate_left(32) | self.counters.last_subject_id as u64
    }
    pub fn new_memo ( &self, subject_id: Option<SubjectId>, parents: MemoRefHead, body: MemoBody) -> MemoRef {

        self.counters.last_memo_id += 1;
        let memo_id = (self.id as u64).rotate_left(32) | self.counters.last_memo_id as u64;

        println!("# Memo.new(id: {},subject_id: {:?}, parents: {:?}, body: {:?})", memo_id, subject_id, parents.memo_ids(), body );

        let memo = Memo {
            id:    memo_id,
            owning_slab_id: self.id,
            subject_id: subject_id,
            inner: Arc::new(MemoInner {
                id:    memo_id,
                subject_id: subject_id,
                parents: parents,
                body: body
            })
        };

        self.memoref_from_memo_and_origin(memo, &MemoOrigin::SameSlab).0
    }
    pub fn new_memo_basic (&self, subject_id: Option<SubjectId>, parents: MemoRefHead, body: MemoBody) -> MemoRef {
        self.new_memo(subject_id, parents, body)
    }
    pub fn new_memo_basic_noparent (&self, subject_id: Option<SubjectId>, body: MemoBody) -> MemoRef {
        self.new_memo(subject_id, MemoRefHead::new(), body)
    }

    pub fn memoref_from_memo_and_origin(&self, memo: Memo, memo_origin: &MemoOrigin ) -> (MemoRef,bool) {
        assert!(memo.owning_slab_id == self.id);

        let peerlist = match memo_origin {
            &MemoOrigin::SameSlab => {
                // my own peering is added at serialization time
                MemoPeerList(vec![])
            }
            &MemoOrigin::OtherSlab(origin_slabref,ref origin_peering_status) => {
                MemoPeerList(vec![MemoPeer{
                    slabref: origin_slabref.clone(),
                    status: origin_peering_status.clone()
                }])
            }
        };

        let (memoref, had_memoref) = self.memoref( memo.id, memo.subject_id, &peerlist );

        let residentized = self.residentize_memoref(&memoref, memo);
        (memoref, residentized)
    }
    pub fn memoref( &self, memo_id: MemoId, subject_id: Option<SubjectId>, peers: &MemoPeerList ) -> (MemoRef, bool) {
        let had_memoref;
        let memoref = match self.memorefs_by_id.entry(memo_id) {
            Entry::Vacant(o)   => {
                let mr = MemoRef {
                    id: memo_id,
                    owning_slab_id: self.id,
                    subject_id: subject_id,
                    shared: Arc::new(Mutex::new(
                        MemoRefShared {
                            id: memo_id,
                            peerlist: *peers.clone(),
                            ptr: MemoRefPtr::Remote
                        }
                    ))
                };

                had_memoref = false;
                o.insert( mr ).clone()// TODO: figure out how to prolong the borrow here & avoid clone
            }
            Entry::Occupied(o) => {
                let mr = o.get();
                had_memoref = true;
                mr.apply_peers( peers );
                mr.clone()
            }
        };

        (memoref, had_memoref)
    }
    pub fn residentize_memoref(&self, memoref: &MemoRef, memo: Memo) -> bool {
        println!("# MemoRef({}).residentize()", self.id);

        assert!(memoref.owning_slab_id == self.id);
        assert!( memoref.id == memo.id );

        // TODO: get rid of mutex here
        let mut inner = memoref.inner();

        if let MemoRefPtr::Remote = inner.ptr {
            inner.ptr = MemoRefPtr::Resident( memo );

            // should this be using do_peering_for_memo?
            // doing it manually for now, because I think we might only want to do
            // a concise update to reflect our peering status change

            let peering_memoref = self.new_memo(
                None,
                MemoRefHead::from_memoref(memoref.clone()),
                MemoBody::Peering(
                    memoref.id,
                    memoref.subject_id,
                    MemoPeerList(vec![ MemoPeer{
                        slabref: self.my_ref,
                        status: MemoPeeringStatus::Resident
                    }])
                )
            );

            for peer in inner.peerlist.0.iter() {
                peer.slabref.send( &self.my_ref, peering_memoref.clone() );
            }

            // residentized
            true
        }else{
            // already resident
            false
        }
    }
    pub fn remotize_memoref( &self, memoref: &MemoRef ) {
        assert!(memoref.owning_slab_id == self.id);

        println!("# MemoRef({}).remotize()", self.id);
        let mut inner = memoref.inner();

        if let MemoRefPtr::Resident(_) = inner.ptr {
            if inner.peerlist.0.len() == 0 {
                panic!("Attempt to remotize a non-peered memo")
            }

            let peering_memoref = self.new_memo_basic(
                None,
                MemoRefHead::from_memoref(memoref.clone()),
                MemoBody::Peering(
                    memoref.id,
                    memoref.subject_id,
                    MemoPeerList(vec![MemoPeer{
                        slabref: self.my_ref,
                        status: MemoPeeringStatus::Participating
                    }])
                )
            );

            for peer in inner.peerlist.0.iter() {
                peer.slabref.send( &self.my_ref, peering_memoref.clone() );
            }
        }

        inner.ptr = MemoRefPtr::Remote;
    }
    pub fn slabref_from_local_slab(&self, peer_slab: &Slab) -> SlabRef {

        let args = TransmitterArgs::Local(&peer_slab);
        let presence = SlabPresence{
            slab_id: peer_slab.id,
            address: TransportAddress::Local,
            lifetime: SlabAnticipatedLifetime::Unknown
        };

        self.slabref(args, &presence)
    }
    pub fn slabref_from_presence(&self, presence: &SlabPresence) -> Result<SlabRef,&str> {

        match presence.address {
            TransportAddress::Simulator  => {
                return Err("Invalid - Cannot create simulator slabref from presence")
            }
            TransportAddress::Local      => {
                return Err("Invalid - Cannot create local slabref from presence")
            }
            _ => { }
        };

        let args = TransmitterArgs::Remote( &presence.slab_id, &presence.address );

        Ok(self.slabref( args, presence ))
    }
    fn slabref(&self, args: TransmitterArgs, presence: &SlabPresence ) -> SlabRef {

        if let Some(slabref) = self.peer_refs.iter().find(|r| r.0.slab_id == presence.slab_id ) {
            if slabref.apply_presence(presence) {
                let new_trans = self.net.get_transmitter( args ).expect("new_from_slab net.get_transmitter");
                let return_address = self.net.get_return_address( &presence.address ).expect("return address not found");

                *(slabref.0.tx.lock().unwrap()) = new_trans;
                slabref.0.return_address.store(&mut return_address, atomic::Ordering::Relaxed);
            }
            return slabref.clone();
        }else{
            let tx = self.net.get_transmitter( args ).expect("new_from_slab net.get_transmitter");
            let return_address = self.net.get_return_address( &presence.address ).expect("return address not found");

            let inner = SlabRefInner {
                slab_id: presence.slab_id,
                owning_slab_id: self.id, // for assertions only?
                presence: Mutex::new(vec![presence.clone()]),
                tx: Mutex::new(tx),
                return_address: atomic::AtomicPtr::new(&mut return_address),
            };

            let slabref = SlabRef(Arc::new(inner));
            self.peer_refs.push(slabref);
            return slabref;
        };

    }
    pub fn presence_for_origin (&self, origin_slabref: &SlabRef ) -> SlabPresence {
        // Get the address that the remote slab would recogize
        SlabPresence {
            slab_id: self.id,
            address: origin_slabref.get_return_address(),
            lifetime: SlabAnticipatedLifetime::Unknown
        }
    }
    pub fn get_root_index_seed (&self) -> Option<MemoRefHead> {
        self.net.get_root_index_seed()
    }
    pub fn put_memos<'a> (&mut self, memo_origin: &MemoOrigin, memos: Vec<Memo> ) -> Vec<MemoRef> {
        let mids : Vec<MemoId> = memos.iter().map(|x| -> MemoId{ x.id }).collect();
        println!("# SlabShared({}).put_memos({:?},{:?})", self.id, memo_origin, mids );

        // TODO: Evaluate more efficient ways to group these memos by subject
        let mut subject_updates : HashMap<SubjectId, MemoRefHead> = HashMap::new();
        let mut memorefs = Vec::with_capacity( memos.len() );

        // TODO: figure out how to appease the borrow checker without cloning this
        let my_slab = self.get_my_slab();
        let my_ref = my_slab.get_ref().clone();

        for memo in memos {

            self.memos_received += 1;
            println!("# \\ Memo Type {:?}", memo.inner.body );
            // Store the memoref - avoid creating duplicates

            let (memoref, pre_existed) = self.memoref_from_memo_and_origin( &memo, memo_origin );

            if pre_existed {
                self.memos_redundantly_received += 1;
            }
println!("# SlabShared({}).put_memos(A)", self.id);
            match memo_origin {
                &MemoOrigin::SameSlab => {
                    //
                }
                &MemoOrigin::OtherSlab(origin_slabref,ref origin_peering_status) => {
    println!("# SlabShared({}).put_memos(B)", self.id);
                    self.check_memo_waiters(&memo);
        println!("# SlabShared({}).put_memos(C)", self.id);
                    self.handle_memo_from_other_slab(&memo, &memoref, &origin_slabref, origin_peering_status, &my_slab, &my_ref);

                    println!("# SlabShared({}).put_memos(D)", self.id);
                    memoref.update_peer(origin_slabref, origin_peering_status.clone());

                    println!("# SlabShared({}).put_memos(E)", self.id);
                    self.do_peering_for_memo(&memo, &memoref, &origin_slabref, &my_slab, &my_ref);

                    println!("# SlabShared({}).put_memos(F)", self.id);
                    if let Some(subject_id) = memo.subject_id {
                        let mut head = subject_updates.entry( subject_id ).or_insert( MemoRefHead::new() );
                        head.apply_memoref(&memoref, &my_slab);
                    }

        println!("# SlabShared({}).put_memos(G)", self.id);
                }
            };

            //TODO: should we emit all memorefs, or just those with subject_ids?
            memorefs.push(memoref);
        }

        self.emit_memos(&memorefs);

        for (subject_id,head) in subject_updates {
            self.dispatch_subject_head(subject_id, &head);
        }

        memorefs
    }
    pub fn dispatch_subject_head (&self, subject_id: u64, head : &MemoRefHead){
        println!("# \t\\ SlabShared({}).dispatch_subject_head({}, {:?})", self.id, subject_id, head.memo_ids() );
        if let Some(subscribers) = self.subject_subscriptions.get( &subject_id ) {
            for weakcontext in subscribers {
                if let Some(context) = weakcontext.upgrade() {
                    context.apply_subject_head( subject_id, head );
                }

            }
        }
    }

    fn check_peering_target( &self, memo: &Memo ) -> u8 {
        if memo.does_peering() {
            5
        }else{
            // This is necessary to prevent memo routing loops for now, as
            // memoref.is_peered_with_slabref() obviously doesn't work for non-peered memos
            // something here should change when we switch to gossip/plumtree, but
            // I'm not sufficiently clear on that at the time of this writing
            0
        }
    }
/*    pub fn memo_durability_score( &self, _memo: &Memo ) -> u8 {
        // TODO: devise durability_score algo
        //       Should this number be inflated for memos we don't care about?
        //       Or should that be a separate signal?

        // Proposed factors:
        // Estimated number of copies in the network (my count = count of first order peers + their counts weighted by: uptime?)
        // Present diasporosity ( my diasporosity score = mean peer diasporosity scores weighted by what? )
        0
    }
*/
}

impl Drop for SlabInner {
    fn drop(&mut self) {
        println!("# Slab({}).drop", self.id);
        // TODO: Drop all observers? Or perhaps observers should drop the slab (weak ref directionality)
    }
}

impl fmt::Debug for Slab {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let shared = self.inner.shared.lock().unwrap();

        fmt.debug_struct("Slab")
            .field("slab_id", &self.id)
            .field("peer_refs", &shared.peer_refs)
            .field("memo_refs", &shared.memorefs_by_id)
            .finish()
    }
}
