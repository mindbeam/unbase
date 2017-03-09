use std::fmt;
use std::sync::mpsc;
use std::sync::mpsc::channel;
use std::sync::{Arc,Mutex,Weak};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use network::SlabRef;

use network::Network;
use memo::*;
use subject::SubjectId;
use memoref::MemoRef;
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

struct SlabShared{
    pub id: SlabId,
    memorefs_by_id: HashMap<MemoId,MemoRef>,
    memo_wait_channels: HashMap<MemoId,Vec<mpsc::Sender<Memo>>>,
    subject_subscriptions: HashMap<SubjectId, Vec<WeakContext>>,

    my_slab: Option<WeakSlab>,
    my_ref: Option<SlabRef>,
    peer_refs: Vec<SlabRef>,
    net: Network
}
struct SlabCounters{
    last_memo_id: u32,
    last_subject_id: u32,
}
struct SlabInner {
    pub id: SlabId,
    my_ref: Mutex<Option<SlabRef>>,
    shared: Mutex<SlabShared>,
    counters: Mutex<SlabCounters>
}

#[derive(Clone)]
pub struct WeakSlab{
    pub id: u32,
    inner: Weak<SlabInner>
}

#[derive(Debug)]
pub enum MemoOrigin<'a>{
    Local,
    Remote(&'a SlabRef)
}

impl Slab {
    pub fn new(net: &Network) -> Slab {
        let slab_id = net.generate_slab_id();

        let shared = SlabShared {
            id: slab_id,
            memorefs_by_id:        HashMap::new(),
            memo_wait_channels:    HashMap::new(),
            subject_subscriptions: HashMap::new(),
            my_slab: None,
            my_ref: None,
            peer_refs: Vec::new(),
            net: net.clone()
        };

        let me = Slab {
            id: slab_id,
            inner: Arc::new(SlabInner {
                id: slab_id,
                my_ref: Mutex::new(None),
                shared: Mutex::new(shared),
                counters: Mutex::new(SlabCounters {
                    last_memo_id: 5000,
                    last_subject_id: 0,
                }),
            })
        };

        let my_ref = net.register_slab(&me);
        {
            *(me.inner.my_ref.lock().unwrap()) = Some(my_ref.clone());
        }

        // not sure if there's a better way to do this, but I want to have a handy return address
        {
            let mut shared = me.inner.shared.lock().unwrap();
            shared.my_slab = Some(me.weak());
            shared.my_ref   = Some(my_ref);
        }

        me
    }
    pub fn weak (&self) -> WeakSlab {
        WeakSlab {
            id: self.id,
            inner: Arc::downgrade(&self.inner)
        }
    }
    pub fn get_ref(&self) -> SlabRef {
        // TODO: figure out how to get this to return a borrow, rather than cloning
        // TODO: determine a better way than a Mutex<Option<SlabRef>> it's dumb
        let my_ref : SlabRef = self.inner.my_ref.lock().unwrap().clone().unwrap();
        my_ref
    }
    pub fn generate_root_index_seed(&self) -> MemoRefHead {

        let memo = Memo::new_basic_noparent(
            self.gen_memo_id(),
            self.generate_subject_id(),
            MemoBody::FullyMaterialized {v: HashMap::new(), r: HashMap::new() } // TODO: accept relations
        );

        let memorefs = self.put_memos(MemoOrigin::Local, vec![ memo.clone() ], true);

        MemoRefHead::from_memoref(memorefs[0].clone())
    }
    pub fn get_root_index_seed (&self) -> MemoRefHead {
        let net;
        {
            let shared = self.inner.shared.lock().unwrap();
            net = shared.net.clone();
        }

        net.get_root_index_seed( self )
    }
    pub fn generate_subject_id(&self) -> SubjectId {
        let mut counters = self.inner.counters.lock().unwrap();
        counters.last_subject_id += 1;

        (self.id as u64).rotate_left(32) | counters.last_subject_id as u64
    }
    pub fn gen_memo_id (&self) -> MemoId {
        let mut counters = self.inner.counters.lock().unwrap();
        counters.last_memo_id += 1;

        (self.id as u64).rotate_left(32) | counters.last_memo_id as u64
    }
    pub fn put_memos(&self, from: MemoOrigin, memos : Vec<Memo>, deliver_local: bool ) -> Vec<MemoRef> {
        if memos.len() == 0 { return Vec::new() }
        let mut shared = self.inner.shared.lock().unwrap();

        shared.put_memos(from, memos, deliver_local)
    }
    pub fn count_of_memorefs_resident( &self ) -> u32 {
        let shared = self.inner.shared.lock().unwrap();
        shared.memorefs_by_id.len() as u32
    }
    pub fn inject_peer_slabref (&self, new_peer_ref: SlabRef ) {
        // We don't have to figure it out, it's just being given to us
        // What luxury!

        let mut shared = self.inner.shared.lock().unwrap();
        shared.peer_refs.push(new_peer_ref);
    }
    pub fn _add_peer_from_memo (&self, slab_id: SlabId ) {
        // TODO - switch peer-injection to use Memos
        //        Identify resident / nonresident Slab

        let mut shared = self.inner.shared.lock().unwrap();

        // check with the network to see if there's an existing slabref
        // This is important for
        //   A. procuring Resident slabrefs, which are otherwise not obtainable
        //   B. sharing slabrefs when possible to increase efficiency
        if let Some(peer_slabref) =  shared.net.get_slabref( slab_id ) {
            shared.peer_refs.push(peer_slabref);
        }
    }
    pub fn peer_slab_count (&self) -> usize {
        let shared = self.inner.shared.lock().unwrap();
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
    pub fn get_my_slab <'a> (&self) -> Slab {
        if let Some(ref weak) = self.my_slab {
            if let Some(s) = weak.upgrade() {
                return s;
            }else{
                panic!("Weak self slab failed to upgrade - this should not happen");
            }
        }else{
            panic!("Called get_my_ref on unregistered slab");
        }
    }
    pub fn get_my_ref (&self) -> &SlabRef {
        if let Some(ref slabref) = self.my_ref {
            &slabref
        }else{
            panic!("Invalid state - Missing my_ref")
        }
    }
    pub fn put_memos<'a> (&mut self, from: MemoOrigin, memos: Vec<Memo>, deliver_local: bool ) -> Vec<MemoRef> {
        let mids : Vec<MemoId> = memos.iter().map(|x| -> MemoId{ x.id }).collect();
        println!("# SlabShared({}).put_memos({:?},{:?})", self.id, from, mids);

        // TODO: Evaluate more efficient ways to group these memos by subject

        let mut subject_updates : HashMap<SubjectId, MemoRefHead> = HashMap::new();
        let mut memorefs = Vec::with_capacity( memos.len() );

        // TODO: figure out how to appease the borrow checker without cloning this
        let my_slab = self.get_my_slab();
        let my_ref = my_slab.get_ref().clone();

        for memo in memos {

            println!("# \\ Memo Type {:?}", memo.inner.body );
            // Store the memoref - avoid creating duplicates
            let memoref = match self.memorefs_by_id.entry(memo.id) {
                Entry::Vacant(o)   => {
                    o.insert( MemoRef::new_from_memo(&memo) ).clone() // TODO: figure out how to prolong the borrow here & avoid clone
                }
                Entry::Occupied(o) => {
                    let mr = o.get();
                    mr.residentize(&my_slab, &memo);
                    // TODO: consider whether all of the below code should be short circuited
                    //       in the case that we already have this memo resident

                    mr.clone()
                }
            };

            match self.memo_wait_channels.entry(memo.id) {
                Entry::Occupied(o) => {
                    for channel in o.get() {
                        // we don't care if it worked or not.
                        // if the channel is closed, we're scrubbing it anyway
                        channel.send(memo.clone()).ok();
                    }
                    o.remove();
                },
                Entry::Vacant(_) => {}
            };

            match memo.inner.body {
                // This Memo is a peering status update for another memo
                MemoBody::Peering(memo_id, ref slabref, ref status) => {
                    // Don't peer with yourself
                    if slabref.slab_id != my_ref.slab_id {
                        // TODO: Determine when this memo is superseded/stale, punt update
                        let peered_memoref = self.memorefs_by_id.entry(memo_id).or_insert_with(|| MemoRef::new_remote(memo_id));
                        peered_memoref.update_peer(slabref, status);
                    }
                },
                MemoBody::MemoRequest(ref desired_memo_ids, ref requesting_slabref ) => {

                    if requesting_slabref.slab_id != my_ref.slab_id {
                        for desired_memo_id in desired_memo_ids {
                            if let Some(desired_memoref) = self.memorefs_by_id.get(&desired_memo_id) {
                                if let Some(desired_memo) = desired_memoref.get_memo_if_resident() {
                                    requesting_slabref.send_memo(&my_ref, desired_memo)
                                } else {
                                    // Somebody asked me for a memo I don't have
                                    // It would be neighborly to tell them I don't have it
                                    let peering_memo = Memo::new(
                                        my_slab.gen_memo_id(), 0,
                                        MemoRefHead::from_memoref(memoref.clone()),
                                        MemoBody::Peering( memo.id, my_ref.clone(), PeeringStatus::Participating)
                                    );
                                    requesting_slabref.send_memo(&my_ref, peering_memo)
                                }
                            }else{
                                let peering_memo = Memo::new(
                                    my_slab.gen_memo_id(), 0,
                                    MemoRefHead::from_memoref(memoref.clone()),
                                    MemoBody::Peering( memo.id, my_ref.clone(), PeeringStatus::NonParticipating)
                                );
                                requesting_slabref.send_memo(&my_ref, peering_memo)
                            }
                        }
                    }
                }
                //MemoBody::Relation(ref relations) => {
                //
                //},
                _ => {}
            }
            // Peering memos don't get peering memos, but Edit memos do
            // Abstracting this, because there might be more types that don't do peering
            if memo.does_peering() {
                // That we received the memo means that the sender didn't think we had it
                // Whether or not we had it already, lets tell them we have it now.
                // It's useful for them to know we have it, and it'll help them STFU
                if let MemoOrigin::Remote(origin_slabref) = from {
                    // TODO: Don't assume that receiving it from origin_slabref means we should assume it to be resident there
                    // Should EITHER: Ensure that a peering memo is co-delivered, OR ensure the memo is delivered with PeeringStatus
                    // memoref.update_peer(origin_slabref, &PeeringStatus::Resident);

                    // TODO: determine if peering memo should:
                    //    A. use parents at all
                    //    B. and if so, what should be should we be using them for?
                    //    C. Should we be sing that to determine the peered memo instead of the payload?
                    let peering_memo = Memo::new(
                        my_slab.gen_memo_id(), 0,
                        MemoRefHead::from_memoref(memoref.clone()),
                        MemoBody::Peering( memo.id, my_ref.clone(), PeeringStatus::Resident)
                    );
                    origin_slabref.send_memo( &my_ref, peering_memo );
                }
            }

            if memo.subject_id > 0 {
                if deliver_local {
                    let mut head = subject_updates.entry( memo.subject_id ).or_insert( MemoRefHead::new() );
                    head.apply_memoref(&memoref, &my_slab);
                }
                //TODO: should we emit all memorefs, or just those with subject_ids?
                memorefs.push(memoref);
            }
        }


        // TODO: test each memo for durability_score and emit accordingly
        self.emit_memos(&memorefs);

        if deliver_local {
            for (subject_id,head) in subject_updates {
                self.dispatch_subject_head(subject_id, &head);
            }
        }

        memorefs
    }
    pub fn dispatch_subject_head (&self, subject_id: u64, head : &MemoRefHead){
        println!("# \t\\ Slab({}).dispatch_subject_head({}, {:?})", self.id, subject_id, head.memo_ids() );
        if let Some(subscribers) = self.subject_subscriptions.get( &subject_id ) {
            for weakcontext in subscribers {
                if let Some(context) = weakcontext.upgrade() {
                    context.apply_subject_head( &subject_id, head );
                }

            }
        }
    }
    pub fn emit_memos(&self, memorefs: &Vec<MemoRef>) {

        // TODO - configurably auto-deliver these memos
        //        punting for now, because we want the test suite to monkey with delivery


        let my_ref : &SlabRef = self.get_my_ref();
        for memoref in memorefs.iter() {
            if let Some(memo) = memoref.get_memo_if_resident() {
                let needs_peers = self.check_peering_target(&memo);

                for peer_ref in self.peer_refs.iter().filter(|x| !memoref.is_peered_with_slabref(x) ).take( needs_peers as usize ) {
                    println!("# Slab({}).emit_memos - EMIT Memo {} to Slab {}", my_ref.slab_id, memo.id, peer_ref.slab_id );
                    peer_ref.send_memo( my_ref, memo.clone() );
                }
            }
        }

    }

    fn check_peering_target( &self, _memo: &Memo ) -> u8 {
        5
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
