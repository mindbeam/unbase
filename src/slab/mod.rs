mod common_structs;
mod memo;
mod slabref;
mod memoref;

pub use self::common_structs::*;
pub use self::slabref::{SlabRef,SlabRefInner};
pub use self::memoref::{MemoRef,MemoRefInner,MemoRefPtr};
pub use self::memo::{MemoId,Memo,MemoInner,MemoBody};
pub use self::memoref::serde as memoref_serde;
pub use self::memo::serde as memo_serde;

use subject::SubjectId;
use memorefhead::*;
use context::{Context,WeakContext};
use network::{Network,Transmitter,TransmitterArgs,TransportAddress};

use std::ops::Deref;
use std::sync::{Arc,Weak,RwLock,Mutex};
use std::sync::mpsc;
use std::sync::mpsc::channel;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt;

pub type SlabId = u32;

#[derive(Clone)]
pub struct Slab(Arc<SlabInner>);

impl Deref for Slab {
    type Target = SlabInner;
    fn deref(&self) -> &SlabInner {
        &*self.0
    }
}

pub struct SlabInner{
    pub id: SlabId,
    memorefs_by_id: RwLock<HashMap<MemoId,MemoRef>>,
    memo_wait_channels: Mutex<HashMap<MemoId,Vec<mpsc::Sender<Memo>>>>, // TODO: HERE HERE HERE - convert to per thread wait channel senders?
    subject_subscriptions: RwLock<HashMap<SubjectId, Vec<WeakContext>>>,

    counters: RwLock<SlabCounters>,

    pub my_ref: SlabRef,
    peer_refs: RwLock<Vec<SlabRef>>,
    net: Network
}

struct SlabCounters{
    last_memo_id: u32,
    last_subject_id: u32,
    memos_received: u64,
    memos_redundantly_received: u64,
}

impl fmt::Debug for Slab {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {

        fmt.debug_struct("Slab")
            .field("slab_id", &self.id)
            .field("peer_refs", &self.peer_refs)
            .field("memo_refs", &self.memorefs_by_id)
            .finish()
    }
}

#[derive(Clone)]
pub struct WeakSlab{
    pub id: u32,
    inner: Weak<SlabInner>
}
impl WeakSlab {
    pub fn upgrade (&self) -> Option<Slab> {
        match self.inner.upgrade() {
            Some(i) => Some( Slab(i) ),
            None    => None
        }
    }
}

impl Slab {
    pub fn weak (&self) -> WeakSlab {
        WeakSlab {
            id: self.id,
            inner: Arc::downgrade(&self.0)
        }
    }
    pub fn new(net: &Network) -> Slab {
        let slab_id = net.generate_slab_id();

        let my_ref_inner = SlabRefInner {
            slab_id: slab_id,
            owning_slab_id: slab_id, // I own my own ref to me, obviously
            presence: RwLock::new(vec![]), // this bit is just for show
            tx: Mutex::new(Transmitter::new_blackhole()),
            return_address: RwLock::new(TransportAddress::Local),
        };

        let my_ref = SlabRef(Arc::new(my_ref_inner));

        let inner = SlabInner {
            id: slab_id,
            memorefs_by_id:        RwLock::new(HashMap::new()),
            memo_wait_channels:    Mutex::new(HashMap::new()),
            subject_subscriptions: RwLock::new(HashMap::new()),

            counters: RwLock::new(SlabCounters {
                last_memo_id: 5000,
                last_subject_id: 0,
                memos_received: 0,
                memos_redundantly_received: 0,
            }),

            my_ref: my_ref,
            peer_refs: RwLock::new(Vec::new()),
            net: net.clone()
        };

        let me = Slab(Arc::new(inner));

        net.register_local_slab(&me);
        net.conditionally_generate_root_index_seed(&me);

        me
    }

    pub fn count_of_memorefs_resident( &self ) -> u32 {
        self.memorefs_by_id.read().unwrap().len() as u32
    }
    pub fn count_of_memos_received( &self ) -> u64 {
        self.counters.read().unwrap().memos_received as u64
    }
    pub fn count_of_memos_reduntantly_received( &self ) -> u64 {
        self.counters.read().unwrap().memos_redundantly_received as u64
    }
    pub fn peer_slab_count (&self) -> usize {
        self.peer_refs.read().unwrap().len() as usize
    }
    pub fn create_context (&self) -> Context {
        Context::new(self)
    }
    pub fn subscribe_subject (&self, subject_id: u64, context: &Context) {
        let weakcontext : WeakContext = context.weak();

        match self.subject_subscriptions.write().unwrap().entry(subject_id){
            Entry::Occupied(mut e) => {
                e.get_mut().push(weakcontext)
            }
            Entry::Vacant(e) => {
                e.insert(vec![weakcontext]);
            }
        }
        return;
    }
    pub fn unsubscribe_subject (&self,  subject_id: u64, context: &Context ){
        if let Some(subs) = self.subject_subscriptions.write().unwrap().get_mut(&subject_id) {
            let weak_context = context.weak();
            subs.retain(|c| {
                c.cmp(&weak_context)
            });
            return;
        }
    }
    pub fn memo_wait_channel (&self, memo_id: MemoId ) -> mpsc::Receiver<Memo> {
        let (tx, rx) = channel::<Memo>();

        match self.memo_wait_channels.lock().unwrap().entry(memo_id) {
            Entry::Vacant(o)       => { o.insert( vec![tx] ); }
            Entry::Occupied(mut o) => { o.get_mut().push(tx); }
        };

        rx
    }
    pub fn remotize_memo_ids( &self, memo_ids: &[MemoId] ) -> Result<(),String>{
        println!("# Slab({}).remotize_memo_ids({:?})", self.id, memo_ids);

        let mut memorefs : Vec<MemoRef> = Vec::with_capacity(memo_ids.len());

        {
            let memorefs_by_id = self.memorefs_by_id.read().unwrap();
            for memo_id in memo_ids.iter() {
                if let Some(memoref) = memorefs_by_id.get(memo_id) {
                    memorefs.push( memoref.clone() )
                }
            }
        }

        for memoref in memorefs {
            self.remotize_memoref(&memoref)?;
        }

        Ok(())
    }
    pub fn generate_subject_id(&self) -> SubjectId {
        let mut counters = self.counters.write().unwrap();
        counters.last_subject_id += 1;
        (self.id as u64).rotate_left(32) | counters.last_subject_id as u64
    }
    pub fn new_memo ( &self, subject_id: Option<SubjectId>, parents: MemoRefHead, body: MemoBody) -> MemoRef {
        let mut counters = self.counters.write().unwrap();
        counters.last_memo_id += 1;
        let memo_id = (self.id as u64).rotate_left(32) | counters.last_memo_id as u64;

        println!("# Memo.new(id: {},subject_id: {:?}, parents: {:?}, body: {:?})", memo_id, subject_id, parents.memo_ids(), body );

        let memo = Memo::new(MemoInner {
            id:    memo_id,
            owning_slab_id: self.id,
            subject_id: subject_id,
            parents: parents,
            body: body
        });

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
                MemoPeerList::new(vec![])
            }
            &MemoOrigin::OtherSlab(origin_slabref,ref origin_peering_status) => {
                MemoPeerList::new(vec![MemoPeer{
                    slabref: origin_slabref.clone(),
                    status: origin_peering_status.clone()
                }])
            }
        };

        self.memoref_from_memo_and_peerlist( memo, peerlist )
    }
    pub fn memoref_from_memo_and_peerlist ( &self, memo: Memo, peerlist: MemoPeerList ) -> (MemoRef,bool){
        let had_memoref;
        let memoref = match self.memorefs_by_id.write().unwrap().entry(memo.id) {
            Entry::Vacant(o)   => {
                let mr = MemoRef(Arc::new(
                    MemoRefInner {
                        id: memo.id,
                        owning_slab_id: self.id,
                        subject_id: memo.subject_id,
                        peerlist: RwLock::new(peerlist),
                        ptr:      RwLock::new(MemoRefPtr::Resident(memo))
                    }
                ));

                had_memoref = false;
                o.insert( mr ).clone()// TODO: figure out how to prolong the borrow here & avoid clone
            }
            Entry::Occupied(o) => {
                let mr = o.get();
                had_memoref = true;
                *mr.ptr.write().unwrap() = MemoRefPtr::Resident(memo);
                mr.apply_peers( &peerlist );
                mr.clone()
            }
        };

        (memoref, had_memoref)
    }
    pub fn memoref( &self, memo_id: MemoId, subject_id: Option<SubjectId>, peerlist: MemoPeerList ) -> (MemoRef, bool) {
        let had_memoref;
        let memoref = match self.memorefs_by_id.write().unwrap().entry(memo_id) {
            Entry::Vacant(o)   => {
                let mr = MemoRef(Arc::new(
                    MemoRefInner {
                        id: memo_id,
                        owning_slab_id: self.id,
                        subject_id: subject_id,
                        peerlist: RwLock::new(peerlist),
                        ptr:      RwLock::new(MemoRefPtr::Remote)
                    }
                ));

                had_memoref = false;
                o.insert( mr ).clone()// TODO: figure out how to prolong the borrow here & avoid clone
            }
            Entry::Occupied(o) => {
                let mr = o.get();
                had_memoref = true;
                mr.apply_peers( &peerlist );
                mr.clone()
            }
        };

        (memoref, had_memoref)
    }
    pub fn residentize_memoref(&self, memoref: &MemoRef, memo: Memo) -> bool {
        println!("# MemoRef({}).residentize()", self.id);

        assert!(memoref.owning_slab_id == self.id);
        assert!( memoref.id == memo.id );

        let mut ptr = memoref.ptr.write().unwrap();

        if let MemoRefPtr::Remote = *ptr {
            *ptr = MemoRefPtr::Resident( memo );

            // should this be using do_peering_for_memo?
            // doing it manually for now, because I think we might only want to do
            // a concise update to reflect our peering status change

            let peering_memoref = self.new_memo(
                None,
                MemoRefHead::from_memoref(memoref.clone()),
                MemoBody::Peering(
                    memoref.id,
                    memoref.subject_id,
                    MemoPeerList::new(vec![ MemoPeer{
                        slabref: self.my_ref.clone(),
                        status: MemoPeeringStatus::Resident
                    }])
                )
            );

            for peer in memoref.peerlist.read().unwrap().iter() {
                peer.slabref.send( &self.my_ref, &peering_memoref );
            }

            // residentized
            true
        }else{
            // already resident
            false
        }
    }
    pub fn remotize_memoref( &self, memoref: &MemoRef ) -> Result<(),String> {
        assert!(memoref.owning_slab_id == self.id);

        println!("# MemoRef({}).remotize()", self.id);

        let mut ptr = memoref.ptr.write().unwrap();

        if let MemoRefPtr::Resident(_) = *ptr {
            let peerlist = memoref.peerlist.read().unwrap();

            if peerlist.len() == 0 {
                return Err("Cannot remotize a zero-peer memo".to_string());
            }

            let peering_memoref = self.new_memo_basic(
                None,
                MemoRefHead::from_memoref(memoref.clone()),
                MemoBody::Peering(
                    memoref.id,
                    memoref.subject_id,
                    MemoPeerList::new(vec![MemoPeer{
                        slabref: self.my_ref.clone(),
                        status: MemoPeeringStatus::Participating
                    }])
                )
            );

            for peer in peerlist.iter() {
                peer.slabref.send( &self.my_ref, &peering_memoref );
            }
        }

        *ptr = MemoRefPtr::Remote;

        Ok(())
    }
    pub fn request_memo (&self, memoref: &MemoRef) -> u8 {

        let request_memo = self.new_memo_basic(
            None,
            MemoRefHead::new(), // TODO: how should this be parented?
            MemoBody::MemoRequest(
                vec![memoref.id],
                self.my_ref.clone()
            )
        );

        let mut sent = 0u8;
        for peer in memoref.peerlist.read().unwrap().iter().take(5) {
            peer.slabref.send( &self.my_ref, &request_memo.clone() );
            sent += 1;
        }

        sent
    }
    pub fn slabref_from_local_slab(&self, peer_slab: &Self) -> SlabRef {

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

        if let Some(slabref) = self.peer_refs.read().unwrap().iter().find(|r| r.0.slab_id == presence.slab_id ) {
            if slabref.apply_presence(presence) {
                let new_trans = self.net.get_transmitter( args ).expect("new_from_slab net.get_transmitter");
                let return_address = self.net.get_return_address( &presence.address ).expect("return address not found");

                *slabref.0.tx.lock().unwrap() = new_trans;
                *slabref.0.return_address.write().unwrap() = return_address;
            }
            return slabref.clone();
        }else{
            let tx = self.net.get_transmitter( args ).expect("new_from_slab net.get_transmitter");
            let return_address = self.net.get_return_address( &presence.address ).expect("return address not found");

            let inner = SlabRefInner {
                slab_id: presence.slab_id,
                owning_slab_id: self.id, // for assertions only?
                presence: RwLock::new(vec![presence.clone()]),
                tx: Mutex::new(tx),
                return_address: RwLock::new(return_address),
            };

            let slabref = SlabRef(Arc::new(inner));
            self.peer_refs.write().unwrap().push(slabref.clone());

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
    pub fn put_memo (&self, memo_origin: &MemoOrigin, memo: Memo ) -> MemoRef {
        let (memoref, pre_existed) = self.memoref_from_memo_and_origin( memo, memo_origin );

        {
            let mut counters = self.counters.write().unwrap();
            counters.memos_received += 1;
            if pre_existed {
                counters.memos_redundantly_received += 1;
            }
        }

        self.handle_memoref( memo_origin, &memoref ); // located in memohandling.rs
        if let Some(subject_id) = memoref.subject_id {
            self.dispatch_subject_head( subject_id, &memoref.to_head());
        }

        memoref
    }
    pub fn put_memos(&self, memo_origin: &MemoOrigin, mut memos: Vec<Memo> ) -> Vec<MemoRef>{

        // TODO: Evaluate more efficient ways to group these memos by subject
        let mut subject_updates : HashMap<SubjectId, MemoRefHead> = HashMap::new();
        let mut memorefs = Vec::with_capacity( memos.len() );
        let mut pre_existing = 0u64;

        for memo in memos.drain(..){
            let (memoref, pre_existed) = self.memoref_from_memo_and_origin( memo, memo_origin );
            if pre_existed { pre_existing += 1 }

            self.handle_memoref( memo_origin, &memoref ); // located in memohandling.rs

            if let Some(subject_id) = memoref.subject_id {
                let mut head = subject_updates.entry( subject_id ).or_insert( MemoRefHead::new() );
                head.apply_memoref(&memoref, self);
            }

            memorefs.push(memoref);
        }

        {
            let mut counters = self.counters.write().unwrap();
            counters.memos_received += memorefs.len() as u64;
            counters.memos_redundantly_received += pre_existing;
        }

        for (subject_id,head) in subject_updates {
            self.dispatch_subject_head(subject_id, &head);
        }

        memorefs
    }
    pub fn dispatch_subject_head (&self, subject_id: u64, head : &MemoRefHead){
        println!("# \t\\ SlabShared({}).dispatch_subject_head({}, {:?})", self.id, subject_id, head.memo_ids() );
        if let Some(subscribers) = self.subject_subscriptions.read().unwrap().get( &subject_id ) {
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
    pub fn handle_memoref (&self, memo_origin: &MemoOrigin, memoref: &MemoRef ){
        println!("# SlabShared({}).handle_memoref({:?},{:?})", self.id, memo_origin, memoref.id);

        match memo_origin {
            &MemoOrigin::SameSlab => {
                //do we want to do anything here?
            }
            &MemoOrigin::OtherSlab(origin_slabref,ref origin_peering_status) => {
                if let Some(ref memo) = memoref.get_memo_if_resident() {
                    self.check_memo_waiters(memo);
                    self.handle_memo_from_other_slab( memo, &memoref, &origin_slabref, origin_peering_status);
                    self.do_peering(&memoref, &origin_slabref);

                }
                memoref.update_peer(origin_slabref, origin_peering_status.clone());
            }
        }

        self.consider_emit_memo(&memoref);

    }
    pub fn check_memo_waiters ( &self, memo: &Memo) {
        match self.memo_wait_channels.lock().unwrap().entry(memo.id) {
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
    }
    pub fn handle_memo_from_other_slab( &self, memo: &Memo, memoref: &MemoRef, origin_slabref: &SlabRef, _origin_peering_status: &MemoPeeringStatus ){

        match memo.body {
            // This Memo is a peering status update for another memo
            MemoBody::SlabPresence{ p: ref presence, r: ref opt_root_index_seed } => {

                let should_process;
                match opt_root_index_seed {
                    &Some(ref root_index_seed) => {
                        // HACK - this should be done inside the deserialize
                        for memoref in root_index_seed.iter() {
                            memoref.update_peer(origin_slabref, MemoPeeringStatus::Resident);
                        }

                        should_process = self.net.apply_root_index_seed( &presence, root_index_seed );
                    }
                    &None => {
                        should_process = true;
                    }
                }

                if should_process {
                    if let Ok(_mentioned_slabref) = self.slabref_from_presence( presence ) {
                        // TODO: should we be telling the origin slabref, or the presence slabref that we're here?
                        //       these will usually be the same, but not always

                        let my_presence_memoref = self.new_memo_basic(
                            None,
                            memoref.to_head(),
                            MemoBody::SlabPresence{
                                p: self.presence_for_origin( origin_slabref ),
                                r: self.get_root_index_seed()
                            }
                        );

                        origin_slabref.send( &self.my_ref, &my_presence_memoref );

                    }
                }
            }
            MemoBody::Peering(memo_id, subject_id, ref peerlist ) => {
                let (peered_memoref,_had_memo) = self.memoref( memo_id, subject_id, peerlist.clone() );

                // Don't peer with yourself
                for peer in peerlist.iter().filter(|p| p.slabref.0.slab_id != self.id ) {
                    peered_memoref.update_peer( &peer.slabref, peer.status.clone());
                }
            },
            MemoBody::MemoRequest(ref desired_memo_ids, ref requesting_slabref ) => {

                if requesting_slabref.0.slab_id != self.id {
                    for desired_memo_id in desired_memo_ids {
                        if let Some(desired_memoref) = self.memorefs_by_id.read().unwrap().get(&desired_memo_id) {

                            if desired_memoref.is_resident() {
                                requesting_slabref.send(&self.my_ref, desired_memoref)
                            } else {
                                // Somebody asked me for a memo I don't have
                                // It would be neighborly to tell them I don't have it
                                self.do_peering(&memoref,requesting_slabref);
                            }
                        }else{
                            let peering_memoref = self.new_memo(
                                None,
                                MemoRefHead::from_memoref(memoref.clone()),
                                MemoBody::Peering(
                                    *desired_memo_id,
                                    None,
                                    MemoPeerList::new(vec![MemoPeer{
                                        slabref: self.my_ref.clone(),
                                        status: MemoPeeringStatus::NonParticipating
                                    }])
                                )
                            );
                            requesting_slabref.send(&self.my_ref, &peering_memoref)
                        }
                    }
                }
            }
            _ => {}
        }
    }
    pub fn do_peering(&self, memoref: &MemoRef, origin_slabref: &SlabRef) {

        let do_send = if let Some(memo) = memoref.get_memo_if_resident(){
            // Peering memos don't get peering memos, but Edit memos do
            // Abstracting this, because there might be more types that don't do peering
            memo.does_peering()
        }else{
            // we're always willing to do peering for non-resident memos
            true
        };

        if do_send {
            // That we received the memo means that the sender didn't think we had it
            // Whether or not we had it already, lets tell them we have it now.
            // It's useful for them to know we have it, and it'll help them STFU

            // TODO: determine if peering memo should:
            //    A. use parents at all
            //    B. and if so, what should be should we be using them for?
            //    C. Should we be sing that to determine the peered memo instead of the payload?
            //println!("MEOW {}, {:?}", my_ref );

            let peering_memoref = self.new_memo(
                None,
                memoref.to_head(),
                MemoBody::Peering(
                    memoref.id,
                    memoref.subject_id,
                    memoref.get_peerlist_for_peer(&self.my_ref, origin_slabref)
                )
            );
            origin_slabref.send( &self.my_ref, &peering_memoref );
        }

    }
    pub fn consider_emit_memo(&self, memoref: &MemoRef) {
        // Emit memos for durability and notification purposes
        // At present, some memos like peering and slab presence are emitted manually.
        // TODO: This will almost certainly have to change once gossip/plumtree functionality is added

        // TODO: test each memo for durability_score and emit accordingly
        if let Some(memo) = memoref.get_memo_if_resident() {
            let needs_peers = self.check_peering_target(&memo);

            for peer_ref in self.peer_refs.read().unwrap().iter().filter(|x| !memoref.is_peered_with_slabref(x) ).take( needs_peers as usize ) {
                println!("# Slab({}).emit_memos - EMIT Memo {} to Slab {}", self.my_ref.0.slab_id, memo.id, peer_ref.0.slab_id );
                peer_ref.send( &self.my_ref, memoref );
            }
        }
    }
}

impl Drop for SlabInner {
    fn drop(&mut self) {
        println!("# SlabInner({}).drop", self.id);
        // TODO: Drop all observers? Or perhaps observers should drop the slab (weak ref directionality)
    }
}
