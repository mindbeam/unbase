use super::SlabId;
use super::common_structs::*;
use super::memo::*;
use super::slabref::*;
use super::memoref::*;

use network::{Network,Transmitter};
use subject::SubjectId;
use memorefhead::*;
use context::{Context,WeakContext};
use network::transport::{TransportAddress,TransmitterArgs};

use std::sync::mpsc;
use std::sync::mpsc::channel;
use std::sync::{Arc,RwLock,RwLockReadGuard,RwLockWriteGuard,Mutex,Weak,MutexGuard};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt;

pub struct SlabInner{
    pub id: SlabId,
    memorefs_by_id: RwLock<HashMap<MemoId,MemoRef>>,
    memo_wait_channels: RwLock<HashMap<MemoId,Vec<mpsc::Sender<Memo>>>>,
    subject_subscriptions: RwLock<HashMap<SubjectId, Vec<WeakContext>>>,

    counters: RwLock<SlabCounters>,

    pub my_ref: SlabRef,
    peer_refs: RwLock<Vec<SlabRef>>,
    net: Network
}
pub struct SlabCounters{
    last_memo_id: u32,
    last_subject_id: u32,
    memos_received: u64,
    memos_redundantly_received: u64,
}

impl SlabInner {
    pub fn count_of_memorefs_resident( &self ) -> u32 {
        self.memorefs_by_id.read().unwrap().len() as u32
    }
    pub fn count_of_memos_received( &self ) -> u64 {
        self.memos_received.read().unwrap().len() as u64
    }
    pub fn count_of_memos_reduntantly_received( &self ) -> u64 {
        self.memos_redundantly_received.read().unwrap().len() as u64
    }
    pub fn peer_slab_count (&self) -> usize {
        self.peer_refs.read().unwrap().len() as usize
    }
    pub fn create_context (&self) -> Context {
        Context::new(self)
    }
    pub fn subscribe_subject (&self, subject_id: u64, context: &Context) {
        let weakcontext : WeakContext = context.weak();

        match self.subject_subscriptions.write().unwrap().entry(&subject_id){
            Entry::Occupied(e) => {
                e.get().push(weakcontext)
            }
            Entry::Vacant(e) => {
                e.insert(subject_id, vec![weakcontext]);
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

        match self.memo_wait_channels.write().entry(memo_id) {
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
                let mr = MemoRef(Arc::new(
                        MemoRefInner {
                            id: memo_id,
                            owning_slab_id: self.id,
                            subject_id: subject_id,
                            peerlist: RwLock::new(*peers.clone()),
                            ptr:      RwLock::new(MemoRefPtr::Remote)
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
                peer.slabref.send( &self.my_ref, &peering_memoref );
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
                peer.slabref.send( &self.my_ref, &peering_memoref );
            }
        }

        inner.ptr = MemoRefPtr::Remote;
    }
    fn request_memo (&self, memoref: &MemoRef) -> u8 {

        let request_memo = self.new_memo_basic(
            None,
            MemoRefHead::new(), // TODO: how should this be parented?
            MemoBody::MemoRequest(
                vec![memoref.id],
                self.my_ref.clone()
            )
        );

        let mut sent = 0u8;
        for peer in memoref.inner().peerlist.0.iter().take(5) {
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

        if let Some(slabref) = self.peer_refs.iter().find(|r| r.0.slab_id == presence.slab_id ) {
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
                presence: Mutex::new(vec![presence.clone()]),
                tx: Mutex::new(tx),
                return_address: RwLock::new(return_address),
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
    pub fn put_memo (&mut self, memo_origin: &MemoOrigin, memo: Memo ) -> MemoRef {
        let memoref = self.put_memo_inner( memo_origin, memo );
        *self.memos_received.write().unwrap() += 1;
        self.dispatch_subject_head(subject_id, &memoref.to_head());
    }
    pub fn put_memos(&self, memo_origin: &MemoOrigin, memos: Vec<Memo> ) -> Vec<MemoRef>{

        // TODO: Evaluate more efficient ways to group these memos by subject
        let mut subject_updates : HashMap<SubjectId, MemoRefHead> = HashMap::new();
        let mut memorefs = Vec::with_capacity( memos.len() );

        for memo in memos.drain(){
            memorefs.push(
                self.put_memo( memo_origin, memo, &subject_updates );
            );

            if let Some(subject_id) = memo.subject_id {
                let mut head = subject_updates.entry( subject_id ).or_insert( MemoRefHead::new() );
                head.apply_memoref(&memoref, &my_slab);
            }
        }

        for (subject_id,head) in subject_updates {
            self.dispatch_subject_head(subject_id, &head);
        }

        memorefs
    }
    pub fn handle_memo (&mut self, memo_origin: &MemoOrigin, memo: Memo ) -> (MemoRef, {
        println!("# SlabShared({}).put_memo({:?},{:?},{:?})", self.id, memo_origin, memo.id, memo.body );


// NOTE LEFT OFF HERE
        let (memoref, pre_existed) = self.memoref_from_memo_and_origin( &memo, memo_origin );

        if pre_existed {
            self.memos_redundantly_received += 1;
        }
        match memo_origin {
            &MemoOrigin::SameSlab => {
                //
            }
            &MemoOrigin::OtherSlab(origin_slabref,ref origin_peering_status) => {
                self.check_memo_waiters(&memo);
                self.handle_memo_from_other_slab(&memo, &memoref, &origin_slabref, origin_peering_status, &my_slab, &my_ref);

                memoref.update_peer(origin_slabref, origin_peering_status.clone());

                self.do_peering_for_memo(&memo, &memoref, &origin_slabref, &my_slab, &my_ref);
            }
        };

        self.consider_emit_memo(&memoref);
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
