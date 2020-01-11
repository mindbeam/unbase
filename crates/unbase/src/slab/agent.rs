use std::{
    collections::hash_map::Entry,
    sync::{Arc,RwLock,Mutex},
};

use tracing::debug;

use crate::slab::{SlabId, MemoRef, MemoBody, Memo, MemoInner, SlabRefInner, MemoRefInner, MemoRefPtr, MemoPeerList, MemoPeeringStatus, MemoId, MemoPeer, SlabPresence, SlabAnticipatedLifetime, RelationSlotSubjectHead};
use crate::slab::state::SlabState;
use crate::network::{SlabRef, TransmitterArgs, Transmitter, TransportAddress};
use crate::Network;
use crate::subject::SubjectId;
use crate::memorefhead::MemoRefHead;
use crate::context::{WeakContext, Context};
use crate::error::PeeringError;

pub struct SlabAgent {
    pub id: SlabId,
    state: RwLock<SlabState>,
    net: Network,
    my_ref: SlabRef
}

impl SlabAgent {
    pub fn new ( net: &Network, my_ref: SlabRef ) -> Self {
        let state = RwLock::new(SlabState::new() );

        SlabAgent {
            id: my_ref.slab_id,
            state: state,
            net: net.clone(),
            my_ref: my_ref
        }
    }
    pub (crate) fn stop (&self){
        let mut state = self.state.write().unwrap();
        state.running = false;
    }
    pub (crate) fn is_running (&self) -> bool {
        let state = self.state.read().unwrap();
        state.running
    }
    // Counters,stats, reporting
    #[allow(unused)]
    pub fn count_of_memorefs_resident( &self ) -> u32 {
        let state = self.state.read().unwrap();
        state.memorefs_by_id.len() as u32
    }
    #[allow(unused)]
    pub fn count_of_memos_received( &self ) -> u64 {
        let state = self.state.read().unwrap();
        state.counters.memos_received as u64
    }
    #[allow(unused)]
    pub fn count_of_memos_reduntantly_received( &self ) -> u64 {
        let state = self.state.read().unwrap();
        state.counters.memos_redundantly_received as u64
    }
    #[allow(unused)]
    pub fn peer_slab_count (&self) -> usize {
        let state = self.state.read().unwrap();
        state.peer_refs.len() as usize
    }

    #[tracing::instrument]
    pub fn new_memo ( &self, subject_id: Option<SubjectId>, parents: MemoRefHead, body: MemoBody) -> MemoRef {
        let memo_id = {
            let mut state = self.state.write().unwrap();
            state.counters.last_memo_id += 1;
            (self.id as u64).rotate_left(32) | state.counters.last_memo_id as u64
        };

        debug!(%memo_id);

        let memo = Memo::new(MemoInner {
            id:    memo_id,
            owning_slab_id: self.id,
            subject_id: subject_id,
            parents: parents,
            body: body
        });

        let (memoref, _had_memoref) = self.assert_memoref(memo.id, memo.subject_id, MemoPeerList(Vec::new()), Some(memo) );
        self.consider_emit_memo(&memoref);

        memoref
    }
    pub fn generate_subject_id (&self) -> SubjectId {

        let mut state = self.state.write().unwrap();
        state.counters.last_subject_id += 1;
        (self.id as u64).rotate_left(32) | state.counters.last_subject_id as u64
    }
    pub fn subscribe_subject (&self, subject_id: u64, context: &Context) {
        let weakcontext : WeakContext = context.weak();

        let mut state = self.state.write().unwrap();

        match state.subject_subscriptions.entry(subject_id){
            Entry::Occupied(mut e) => {
                e.get_mut().push(weakcontext)
            }
            Entry::Vacant(e) => {
                e.insert(vec![weakcontext]);
            }
        }
        return;
    }
    #[tracing::instrument]
    pub fn unsubscribe_subject (&self,  subject_id: u64, context: &Context ){
        let mut state = self.state.write().unwrap();

        if let Some(subs) = state.subject_subscriptions.get_mut(&subject_id) {
            let weak_context = context.weak();
            subs.retain(|c| {
                c.cmp(&weak_context)
            });
            return;
        }
    }
    #[tracing::instrument]
    pub fn consider_emit_memo(&self, memoref: &MemoRef) {
        // Emit memos for durability and notification purposes
        // At present, some memos like peering and slab presence are emitted manually.
        // TODO: This will almost certainly have to change once gossip/plumtree functionality is added

        // TODO: test each memo for durability_score and emit accordingly
        if let Some(memo) = memoref.get_memo_if_resident() {
            let needs_peers = self.check_peering_target(&memo);

            debug!("memo is resident");
            let state = self.state.read().unwrap();
            for peer_ref in state.peer_refs.iter().filter(|x| !memoref.is_peered_with_slabref(x) ).take( needs_peers as usize ) {

                peer_ref.send( &self.my_ref, memoref );
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
    pub fn memo_wait_channel (&self, memo_id: MemoId ) -> futures::channel::oneshot::Receiver<Memo> {
        let (tx, rx) = futures::channel::oneshot::channel();

        // TODO this should be moved to agent
        let mut state = self.state.write().unwrap();
        match state.memo_wait_channels.entry(memo_id) {
            Entry::Vacant(o)       => { o.insert( vec![tx] ); }
            Entry::Occupied(mut o) => { o.get_mut().push(tx); }
        };

        rx
    }
    #[tracing::instrument]
    pub fn check_memo_waiters ( &self, memo: &Memo) {
        let mut state = self.state.write().unwrap();
        match state.memo_wait_channels.entry(memo.id) {
            Entry::Occupied(o) => {
                let (_, v) = o.remove_entry();
                for sender in v {
                    // we don't care if it worked or not.
                    // if the channel is closed, we're scrubbing it anyway
                    sender.send(memo.clone()).ok();
                }
            },
            Entry::Vacant(_) => {}
        };
    }
    #[tracing::instrument]
    pub fn handle_memo_from_other_slab( &self, memo: &Memo, memoref: &MemoRef, origin_slabref: &SlabRef ){

        match memo.body {
            // This Memo is a peering status update for another memo
            MemoBody::SlabPresence{ p: ref presence, r: ref opt_root_index_seed } => {

                match opt_root_index_seed {
                    &Some(ref root_index_seed) => {

                        // HACK - this should be done inside the deserialize
                        for memoref in root_index_seed.iter() {
                            memoref.update_peer(origin_slabref, MemoPeeringStatus::Resident);
                        }

                        self.net.apply_root_index_seed( &presence, root_index_seed, &self.my_ref );
                    }
                    &None => {}
                }

                let mut reply = false;
                if let &None = opt_root_index_seed {
                    reply = true;
                }

                if reply {
                    if let Ok(mentioned_slabref) = self.slabref_from_presence( presence ) {
                        // TODO: should we be telling the origin slabref, or the presence slabref that we're here?
                        //       these will usually be the same, but not always

                        let my_presence_memoref = self.new_memo(
                            None,
                            memoref.to_head(),
                            MemoBody::SlabPresence{
                                p: self.presence_for_origin( origin_slabref ),
                                r: self.net.get_root_index_seed_for_agent(&self)
                            }
                        );

                        origin_slabref.send( &self.my_ref, &my_presence_memoref );

                        let _ = mentioned_slabref;
                        // needs PartialEq
                        //if mentioned_slabref != origin_slabref {
                        //   mentioned_slabref.send( &self.my_ref, &my_presence_memoref );
                        //}
                    }
                }
            }
            MemoBody::Peering(memo_id, subject_id, ref peerlist ) => {
                let (peered_memoref,_had_memo) = self.assert_memoref( memo_id, subject_id, peerlist.clone(), None );

                // Don't peer with yourself
                for peer in peerlist.iter().filter(|p| p.slabref.0.slab_id != self.id ) {
                    peered_memoref.update_peer( &peer.slabref, peer.status.clone());
                }
            },
            MemoBody::MemoRequest(ref desired_memo_ids, ref requesting_slabref ) => {

                if requesting_slabref.0.slab_id != self.id {
                    for desired_memo_id in desired_memo_ids {

                        let maybe_desired_memoref = {
                            let state = self.state.read().unwrap();
                            match state.memorefs_by_id.get(&desired_memo_id){
                                Some(mr) => Some(mr.clone()),
                                None => None
                            }
                        };

                        if let Some(desired_memoref) = maybe_desired_memoref {

                            if desired_memoref.is_resident() {
                                requesting_slabref.send(&self.my_ref, &desired_memoref);
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
    // should this be a function of the slabref rather than the owning slab?
    pub fn presence_for_origin (&self, origin_slabref: &SlabRef ) -> SlabPresence {
        // Get the address that the remote slab would recogize
        SlabPresence {
            slab_id: self.id,
            address: origin_slabref.get_return_address(),
            lifetime: SlabAnticipatedLifetime::Unknown
        }
    }
    pub fn slabref_from_presence(&self, presence: &SlabPresence) -> Result<SlabRef,&str> {
        match presence.address {
            TransportAddress::Simulator  => {
                return Err("Invalid - Cannot create simulator slabref from presence")
            }
            TransportAddress::Local      => {
                return Err("Invalid - Cannot create local slabref from presence")
            }
            _ => {
                //let args = TransmitterArgs::Remote( &presence.slab_id, &presence.address );
                Ok(self.assert_slabref( presence.slab_id, &vec![presence.clone()] ))
            }
        }




    }
    #[tracing::instrument]
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

            let peering_memoref = self.new_memo(
                None,
                memoref.to_head(),
                MemoBody::Peering(
                    memoref.id,
                    memoref.subject_id,
                    memoref.get_peerlist_for_peer(&self.my_ref, Some(origin_slabref.slab_id))
                )
            );
            origin_slabref.send( &self.my_ref, &peering_memoref );
        }

    }
    #[tracing::instrument]
    pub fn recv_memoref (&self, memoref : MemoRef){

        if let Some(subject_id) = memoref.subject_id {

            let maybe_sub : Option<Vec<WeakContext>> = {
                // we want to make sure the lock is released before continuing
                let state = self.state.read().unwrap();
                if let Some(ref s) = state.subject_subscriptions.get( &subject_id ) {
                    Some((*s).clone())
                }else{
                    None
                }
            };

            if let Some(subscribers) = maybe_sub {

                for weakcontext in subscribers {

                    if let Some(mut context) = weakcontext.upgrade() {
                        // TODO - update this to use a (blocking) channel so we don't have to do all this silliness with weakcontext
                        context.apply_head_deferred(memoref.to_head());
                    }
                }
            }

        }
    }
    #[tracing::instrument]
    pub fn localize_slabref(&self, slabref: &SlabRef ) -> SlabRef {
        // For now, we don't seem to care what slabref we're being cloned from, just which one we point to

        // IF this slabref points to the destination slab, then use to_sab.my_ref
        // because we know it exists already, and we're not allowed to assert a self-ref
        if self.id == slabref.slab_id {
            self.my_ref.clone()
        }else{
            //let address = &*self.return_address.read().unwrap();
            //let args = TransmitterArgs::Remote( &self.slab_id, address );
            let presence = {
                slabref.presence.read().unwrap().clone()
            };
            self.assert_slabref( slabref.slab_id, &presence )
        }

    }
    #[tracing::instrument]
    pub fn localize_memorefhead (&self, mrh: &MemoRefHead, from_slabref: &SlabRef, include_memos: bool ) -> MemoRefHead {

        if from_slabref.slab_id == self.my_ref.slab_id {
            mrh.clone()
        }else {
            let head = mrh.head.clone();

            let local_from_slabref = self.localize_slabref(&from_slabref);

            let mut newhead = Vec::with_capacity(head.len());
            for mr in head.into_iter() {
                newhead.push(self.localize_memoref(&mr, &local_from_slabref, include_memos));
            }

            MemoRefHead {
                head: newhead,
                owning_slab_id: self.my_ref.slab_id
            }
        }
    }
    #[tracing::instrument]
    pub fn localize_memoref (&self, memoref: &MemoRef, from_slabref: &SlabRef, include_memo: bool ) -> MemoRef {
//        assert!(from_slabref.owning_slab_id == self.id,"MemoRef clone_for_slab owning slab should be identical");
//        assert!(from_slabref.slab_id != self.id,       "MemoRef clone_for_slab dest slab should not be identical");

        // TODO compare SlabRef pointer address rather than id
        if memoref.owning_slab_id == self.id {
            return (*memoref).clone()
        }

        // Because our from_slabref is already owned by the destination slab, there is no need to do peerlist.clone_for_slab
        let peerlist = memoref.get_peerlist_for_peer(from_slabref, Some(self.id));

        // TODO - reduce the redundant work here. We're basically asserting the memoref twice
        let memoref = self.assert_memoref(
            memoref.id,
            memoref.subject_id,
            peerlist.clone(),
            match include_memo {
                true => match *memoref.ptr.read().unwrap() {
                    MemoRefPtr::Resident(ref m) => Some(self.localize_memo(m, from_slabref, &peerlist)),
                    MemoRefPtr::Remote          => None
                },
                false => None
            }
        ).0;

        memoref
    }
    #[tracing::instrument]
    pub fn localize_memo (&self, memo: &Memo, from_slabref: &SlabRef, peerlist: &MemoPeerList) -> Memo {
        assert!(from_slabref.owning_slab_id == self.id, "Memo clone_for_slab owning slab should be identical");

        // TODO - simplify this
        self.reconstitute_memo(
            memo.id,
            memo.subject_id,
            self.localize_memorefhead(&memo.parents, from_slabref, false),
            self.localize_memobody(&memo.body, from_slabref),
            from_slabref,
            peerlist
        ).0
    }
    #[tracing::instrument]
    pub fn reconstitute_memo ( &self, memo_id: MemoId, subject_id: Option<SubjectId>, parents: MemoRefHead, body: MemoBody, origin_slabref: &SlabRef, peerlist: &MemoPeerList ) -> (Memo,MemoRef,bool){
        // TODO: find a way to merge this with assert_memoref to avoid doing duplicative work with regard to peerlist application

        let memo = Memo::new(MemoInner {
            id:             memo_id,
            owning_slab_id: self.id,
            subject_id:     subject_id,
            parents:        parents,
            body:           body
        });

        let (memoref, had_memoref) = self.assert_memoref(memo.id, memo.subject_id, peerlist.clone(), Some(memo.clone()) );

        {
            let mut state = self.state.write().unwrap();
            state.counters.memos_received += 1;
            if had_memoref {
                state.counters.memos_redundantly_received += 1;
            }
        }

        self.consider_emit_memo(&memoref);

        if let Some(ref memo) = memoref.get_memo_if_resident() {

            self.check_memo_waiters(memo);
            self.handle_memo_from_other_slab(memo, &memoref, &origin_slabref);
            self.do_peering(&memoref, &origin_slabref);

        }

        self.recv_memoref(memoref.clone());

        // TODO: reconcile localize_memoref, reconstitute_memo, and recv_memoref
        (memo, memoref, had_memoref)
    }
    #[tracing::instrument]
    fn localize_memobody(&self, mb: &MemoBody, from_slabref: &SlabRef ) -> MemoBody {
        assert!(from_slabref.owning_slab_id == self.id, "MemoBody clone_for_slab owning slab should be identical");

        match mb {
            &MemoBody::SlabPresence{ ref p, ref r } => {
                MemoBody::SlabPresence{
                    p: p.clone(),
                    r: match r {
                        &Some(ref root_mrh) => {
                            Some(self.localize_memorefhead(root_mrh, from_slabref, true))
                        }
                        &None => None
                    }
                }
            },
            &MemoBody::Relation(ref rssh) => {
                MemoBody::Relation(self.localize_relationslothead(rssh,from_slabref))
            }
            &MemoBody::Edit(ref hm) => {
                MemoBody::Edit(hm.clone())
            }
            &MemoBody::FullyMaterialized{ ref v, ref r } => {
                MemoBody::FullyMaterialized{ v: v.clone(), r: self.localize_relationslothead(r,from_slabref)}
            }
            &MemoBody::PartiallyMaterialized{ ref v, ref r } => {
                MemoBody::PartiallyMaterialized{ v: v.clone(), r: self.localize_relationslothead(r, from_slabref)}
            }
            &MemoBody::Peering(memo_id, subject_id, ref peerlist) => {
                MemoBody::Peering(memo_id,subject_id,  self.localize_peerlist(peerlist))
            }
            &MemoBody::MemoRequest(ref memo_ids, ref slabref) =>{
                MemoBody::MemoRequest(memo_ids.clone(), self.localize_slabref(slabref))
            }
        }
    }
    pub fn localize_peerlist(&self, peerlist: &MemoPeerList) -> MemoPeerList {
        MemoPeerList(peerlist.0
            .iter()
            .map(|p| {
                MemoPeer {
                    slabref: self.localize_slabref(&p.slabref),
                    status: p.status.clone(),
                }
            })
            .collect())
    }
    #[tracing::instrument]
    pub fn localize_relationslothead(&self, rsh: &RelationSlotSubjectHead, from_slabref: &SlabRef) -> RelationSlotSubjectHead {
        // panic!("check here to make sure that peers are being properly constructed for the root_index_seed");
        let new = rsh.0
            .iter()
            .map(|(slot_id, &(subject_id, ref mrh))| {
                (*slot_id, (subject_id, self.localize_memorefhead(mrh, from_slabref,false)))
            })
            .collect();

        RelationSlotSubjectHead(new)
    }
    #[allow(unused)]
    #[tracing::instrument]
    pub fn residentize_memoref(&self, memoref: &MemoRef, memo: Memo) -> bool {

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
    #[allow(unused)]
    #[tracing::instrument]
    pub fn remotize_memoref( &self, memoref: &MemoRef ) -> Result<(),PeeringError> {
        assert!(memoref.owning_slab_id == self.id);

        // TODO: check peering minimums here, and punt if we're below threshold

        let send_peers;
        {
            let mut ptr = memoref.ptr.write().unwrap();
            if let MemoRefPtr::Resident(_) = *ptr {
                let peerlist = memoref.peerlist.read().unwrap();

                if peerlist.len() == 0 {
                    return Err(PeeringError::InsufficientReplicas);
                }
                send_peers = peerlist.clone();
                *ptr = MemoRefPtr::Remote;

            }else{
                return Ok(());
            }
        }

        let peering_memoref = self.new_memo(
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

        //self.consider_emit_memo(&memoref);

        for peer in send_peers.iter() {
            peer.slabref.send( &self.my_ref, &peering_memoref );
        }

        Ok(())
    }
    #[tracing::instrument]
    pub fn assert_memoref( &self, memo_id: MemoId, subject_id: Option<SubjectId>, peerlist: MemoPeerList, memo: Option<Memo>) -> (MemoRef, bool) {

        let had_memoref;
        let memoref = match self.state.write().unwrap().memorefs_by_id.entry(memo_id) {
            Entry::Vacant(o)   => {
                let mr = MemoRef(Arc::new(
                    MemoRefInner {
                        id: memo_id,
                        owning_slab_id: self.id,
                        subject_id: subject_id,
                        peerlist: RwLock::new(peerlist),
                        ptr:      RwLock::new(match memo {
                            Some(m) => {
                                assert!(self.id == m.owning_slab_id);
                                MemoRefPtr::Resident(m)
                            }
                            None    => MemoRefPtr::Remote
                        })
                    }
                ));

                had_memoref = false;
                o.insert( mr ).clone()// TODO: figure out how to prolong the borrow here & avoid clone
            }
            Entry::Occupied(o) => {
                let mr = o.get();
                had_memoref = true;
                if let Some(m) = memo {

                    let mut ptr = mr.ptr.write().unwrap();
                    if let MemoRefPtr::Remote = *ptr {
                        *ptr = MemoRefPtr::Resident(m)
                    }
                }
                mr.apply_peers( &peerlist );
                mr.clone()
            }
        };

        (memoref, had_memoref)
    }
    #[tracing::instrument]
    pub fn assert_slabref(&self, slab_id: SlabId, presence: &[SlabPresence] ) -> SlabRef {

        if slab_id == self.id {
            return self.my_ref.clone();
            // don't even look it up if it's me.
            // We must not allow any third party to edit the peering.
            // Also, my ref won't appeara in the list of peer_refs, because it's not a peer
        }

        let maybe_slabref = {
            // Instead of having to scope our read lock, and getting a write lock later
            // should we be using a single write lock for the full function scope?
            let state = self.state.read().unwrap();
            if let Some(slabref) = state.peer_refs.iter().find(|r| r.0.slab_id == slab_id ){
                Some(slabref.clone())
            }else{
                None
            }
        };

        let slabref : SlabRef;
        if let Some(s) = maybe_slabref {
            slabref = s;
        }else{
            let inner = SlabRefInner {
                slab_id:        slab_id,
                owning_slab_id: self.id, // for assertions only?
                presence:       RwLock::new(Vec::new()),
                tx:             Mutex::new(Transmitter::new_blackhole(slab_id)),
                return_address: RwLock::new(TransportAddress::Blackhole),
            };

            slabref = SlabRef(Arc::new(inner));
            let mut state = self.state.write().unwrap();
            state.peer_refs.push(slabref.clone());
        }

        if slab_id == slabref.owning_slab_id {
            return slabref; // no funny business. You don't get to tell me how to reach me
        }

        for p in presence.iter(){
            assert!(slab_id == p.slab_id, "presence slab_id does not match the provided slab_id");

            let mut _maybe_slab = None;
            let args = if p.address.is_local() {
                // playing silly games with borrow lifetimes.
                // TODO: make this less ugly
                _maybe_slab = self.net.get_slabhandle(p.slab_id);

                if let Some(ref slab) = _maybe_slab {
                    TransmitterArgs::Local(slab)
                }else{
                    continue;
                }
            }else{
                TransmitterArgs::Remote( &p.slab_id, &p.address )
            };
             // Returns true if this presence is new to the slabref
             // False if we've seen this presence already

            if slabref.apply_presence(p) {

                let new_trans = self.net.get_transmitter( &args ).expect("assert_slabref net.get_transmitter");
                let return_address = self.net.get_return_address( &p.address ).expect("return address not found");

                *slabref.0.tx.lock().expect("tx.lock()") = new_trans;
                *slabref.0.return_address.write().expect("return_address write lock") = return_address;
            }
        }

        return slabref;

    }
    #[allow(unused)]
    #[tracing::instrument]
    pub fn remotize_memos(&self, memo_ids: &[MemoId] ) -> Result<(),PeeringError>{
        //TODO accept memoref instead of memoid
        let mut memorefs : Vec<MemoRef> = Vec::with_capacity(memo_ids.len());

        {
            let state = self.state.read().unwrap();
            for memo_id in memo_ids.iter() {
                if let Some(memoref) = state.memorefs_by_id.get(memo_id) {
                    memorefs.push( memoref.clone() )
                }
            }
        }

        for memoref in memorefs {
            self.remotize_memoref(&memoref)?;
        }

        Ok(())
    }
}

impl std::fmt::Debug for SlabAgent {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("Slab")
            .field("state", &self.state.read().unwrap())
            .finish()
    }
}