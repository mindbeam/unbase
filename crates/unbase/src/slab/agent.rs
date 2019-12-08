use std::sync::{Arc,RwLock,Mutex};
use std::collections::hash_map::Entry;

use crate::slab::{SlabId, MemoRef, MemoBody, Memo, MemoInner, SlabRefInner, MemoRefInner, MemoRefPtr, MemoPeerList, MemoPeeringStatus, MemoId, MemoPeer, SlabPresence, SlabAnticipatedLifetime, RelationSlotSubjectHead};
use crate::slab::state::SlabState;
use crate::network::{SlabRef, TransmitterArgs, Transmitter, TransportAddress};
use crate::Network;
use crate::subject::SubjectId;
use crate::memorefhead::{MemoRefHead, ExtMemoRefHead};
use crate::context::WeakContext;

pub struct SlabAgent {
    pub id: SlabId,
    state: RwLock<SlabState>,
    net: Network,
    my_ref: SlabRef,
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
    pub fn new_memo ( &self, subject_id: Option<SubjectId>, parents: MemoRefHead, body: MemoBody) -> MemoRef {
        let memo_id = {
            let mut state = self.state.write().unwrap();
            state.counters.last_memo_id += 1;
            (self.id as u64).rotate_left(32) | state.counters.last_memo_id as u64
        };

        //println!("# Slab({}).new_memo(id: {},subject_id: {:?}, parents: {:?}, body: {:?})", self.id, memo_id, subject_id, parents.memo_ids(), body );

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
    pub fn new_memo_basic (&self, subject_id: Option<SubjectId>, parents: MemoRefHead, body: MemoBody) -> MemoRef {
        self.new_memo(subject_id, parents, body)
    }
    pub fn new_memo_basic_noparent (&self, subject_id: Option<SubjectId>, body: MemoBody) -> MemoRef {
        self.new_memo(subject_id, MemoRefHead::new(), body)
    }
    pub fn consider_emit_memo(&self, memoref: &MemoRef) {
        // Emit memos for durability and notification purposes
        // At present, some memos like peering and slab presence are emitted manually.
        // TODO: This will almost certainly have to change once gossip/plumtree functionality is added

        // TODO: test each memo for durability_score and emit accordingly
        if let Some(memo) = memoref.get_memo_if_resident() {
            let needs_peers = self.check_peering_target(&memo);


            //println!("Slab({}).consider_emit_memo {} - A ({:?})", self.id, memoref.id, &*self.peer_refs.read().unwrap() );
            let state = self.state.read().unwrap();
            for peer_ref in state.peer_refs.iter().filter(|x| !memoref.is_peered_with_slabref(x) ).take( needs_peers as usize ) {

                //println!("# Slab({}).emit_memos - EMIT Memo {} to Slab {}", self.id, memo.id, peer_ref.slab_id );
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
    pub fn check_memo_waiters ( &self, memo: &Memo) {
        let state = self.state.write().unwrap();
        match state.memo_wait_channels.entry(memo.id) {
            Entry::Occupied(o) => {
                for sender in o.get() {
                    // we don't care if it worked or not.
                    // if the channel is closed, we're scrubbing it anyway
                    sender.send(memo.clone()).ok();
                }
                o.remove();
            },
            Entry::Vacant(_) => {}
        };
    }
    pub fn handle_memo_from_other_slab( &self, memo: &Memo, memoref: &MemoRef, origin_slabref: &SlabRef ){
        //println!("Slab({}).handle_memo_from_other_slab({})", self.id, memo.id );

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

                        let my_presence_memoref = self.new_memo_basic(
                            None,
                            memoref.to_head(),
                            MemoBody::SlabPresence{
                                p: self.presence_for_origin( origin_slabref ),
                                r: self.net.get_root_index_seed().map(|seed| self.localize_memorefhead(seed) )
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
                            let state = self.state.write().unwrap();
                            state.memorefs_by_id.get(&desired_memo_id).clone()
                        };

                        if let Some(desired_memoref) = maybe_desired_memoref {

                            if desired_memoref.is_resident() {
                                requesting_slabref.send(&self.my_ref, desired_memoref);
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
                unimplemented!()
            }
        };


        //let args = TransmitterArgs::Remote( &presence.slab_id, &presence.address );

        Ok(self.assert_slabref( presence.slab_id, &vec![presence.clone()] ))
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
                    memoref.get_peerlist_for_peer(&self.my_ref, Some(origin_slabref.slab_id))
                )
            );
            origin_slabref.send( &self.my_ref, &peering_memoref );
        }

    }
    pub fn recv_memoref (&self, memoref : MemoRef){
        //println!("# \t\\ Slab({}).dispatch_memoref({})", self.id, &memoref.id );

        if let Some(subject_id) = memoref.subject_id {

            let maybe_sub : Option<Vec<WeakContext>> = {
                // we want to make sure the lock is released before continuing
                if let Some(ref s) = self.subject_subscriptions.read().unwrap().get( &subject_id ) {
                    Some((*s).clone())
                }else{
                    None
                }
            };

            if let Some(subscribers) = maybe_sub {

                for weakcontext in subscribers {

                    if let Some(context) = weakcontext.upgrade() {

                        context.apply_subject_head( subject_id, &memoref.to_head(), true );
                    }
                }
            }

        }
    }
    pub fn localize_slabref(&self, slabref: &SlabRef ) -> SlabRef {
        // For now, we don't seem to care what slabref we're being cloned from, just which one we point to

        //println!("Slab({}).SlabRef({}).clone_for_slab({})", self.owning_slab_id, self.slab_id, to_slab.id );

        // IF this slabref points to the destination slab, then use to_sab.my_ref
        // because we know it exists already, and we're not allowed to assert a self-ref
        if self.id == slabref.slab_id {
            self.my_ref.clone()
        }else{
            //let address = &*self.return_address.read().unwrap();
            //let args = TransmitterArgs::Remote( &self.slab_id, address );
            self.assert_slabref( slabref.slab_id, &*slabref.presence.read().unwrap() )
        }

    }
    pub fn localize_memorefhead (&self, mrh: &MemoRefHead, from_slabref: &SlabRef, include_memos: bool ) -> MemoRefHead {

        let slabref = self.localize_slabref(from_slabref);
        MemoRefHead( self.iter().map(|mr| self.localize_memoref(mr, from_slabref, include_memos )).collect() )
    }
    pub fn localize_memoref (&self, memoref: &MemoRef, from_slabref: &SlabRef, include_memo: bool ) -> MemoRef {
//        assert!(from_slabref.owning_slab_id == self.id,"MemoRef clone_for_slab owning slab should be identical");
//        assert!(from_slabref.slab_id != self.id,       "MemoRef clone_for_slab dest slab should not be identical");

        // TODO compare SlabRef pointer address rather than id
        if memoref.owning_slab_id != self.id {
            return (*memoref).clone()
        }
        //println!("Slab({}).Memoref.clone_for_slab({})", self.owning_slab_id, self.id);

        // Because our from_slabref is already owned by the destination slab, there is no need to do peerlist.clone_for_slab
        let peerlist = memoref.get_peerlist_for_peer(from_slabref, Some(self.id));
        //println!("Slab({}).Memoref.clone_for_slab({}) C -> {:?}", self.owning_slab_id, self.id, peerlist);

        // TODO - reduce the redundant work here. We're basically asserting the memoref twice
        let memoref = self.assert_memoref(
            memoref.id,
            self.subject_id,
            peerlist.clone(),
            match include_memo {
                true => match *self.ptr.read().unwrap() {
                    MemoRefPtr::Resident(ref m) => Some(self.localize_memo(m, from_slabref, &peerlist)),
                    MemoRefPtr::Remote          => None
                },
                false => None
            }
        ).0;


        //println!("MemoRef.clone_for_slab({},{}) peerlist: {:?} -> MemoRef({:?})", from_slabref.slab_id, to_slab.id, &peerlist, &memoref );

        memoref
    }
    pub fn localize_memo (&self, memo: &Memo, from_slabref: &SlabRef, peerlist: &MemoPeerList) -> Memo {
        assert!(from_slabref.owning_slab_id == self.id, "Memo clone_for_slab owning slab should be identical");

        //println!("Slab({}).Memo.clone_for_slab(memo: {}, from: {}, to: {}, peers: {:?})", self.owning_slab_id, self.id, from_slabref.slab_id, to_slab.id, peerlist );
        self.reconstitute_memo(
            memo.id,
            memo.subject_id,
            self.localize_memorefhead(&memo.parents, from_slabref, false),
            self.localize_memobody(&memo.body, from_slabref),
            from_slabref,
            peerlist
        ).0
    }

    fn localize_memobody(&self, mb: &MemoBody, from_slabref: &SlabRef ) -> MemoBody {
        assert!(from_slabref.owning_slab_id == self.id, "MemoBody clone_for_slab owning slab should be identical");

        match self {
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
    pub fn localize_relationslothead(&self, rsh: &RelationSlotSubjectHead, from_slabref: &SlabRef) -> RelationSlotSubjectHead {
        // HERE HERE HERE TODO
        // panic!("check here to make sure that peers are being properly constructed for the root_index_seed");
        let new = self.0
            .iter()
            .map(|(slot_id, &(subject_id, ref mrh))| {
                (*slot_id, (subject_id, self.localize_memorefhead(mrh, from_slabref,false)))
            })
            .collect();

        RelationSlotSubjectHead(new)
    }
    pub fn residentize_memoref(&self, memoref: &MemoRef, memo: Memo) -> bool {
        //println!("# Slab({}).MemoRef({}).residentize()", self.id, memoref.id);

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

        //println!("# Slab({}).MemoRef({}).remotize()", self.id, memoref.id );

        // TODO: check peering minimums here, and punt if we're below threshold

        let send_peers;
        {
            let mut ptr = memoref.ptr.write().unwrap();
            if let MemoRefPtr::Resident(_) = *ptr {
                let peerlist = memoref.peerlist.read().unwrap();

                if peerlist.len() == 0 {
                    return Err("Cannot remotize a zero-peer memo".to_string());
                }
                send_peers = peerlist.clone();
                *ptr = MemoRefPtr::Remote;

            }else{
                return Ok(());
            }
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

        //self.consider_emit_memo(&memoref);

        for peer in send_peers.iter() {
            peer.slabref.send( &self.my_ref, &peering_memoref );
        }

        Ok(())
    }
    pub fn request_memo (&self, memoref: &MemoRef) -> u8 {
        //println!("Slab({}).request_memo({})", self.id, memoref.id );

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
            //println!("Slab({}).request_memo({}) from {}", self.id, memoref.id, peer.slabref.slab_id );
            peer.slabref.send( &self.my_ref, &request_memo.clone() );
            sent += 1;
        }

        sent
    }
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
    pub fn assert_slabref(&self, slab_id: SlabId, presence: &[SlabPresence] ) -> SlabRef {
        //println!("# Slab({}).assert_slabref({}, {:?})", self.id, slab_id, presence );

        if slab_id == self.id {
            return self.my_ref.clone();
            // don't even look it up if it's me.
            // We must not allow any third party to edit the peering.
            // Also, my ref won't appeara in the list of peer_refs, because it's not a peer
        }

        let maybe_slabref = {
            // Instead of having to scope our read lock, and getting a write lock later
            // should we be using a single write lock for the full function scope?
            if let Some(slabref) = self.peer_refs.read().expect("peer_refs.read()").iter().find(|r| r.0.slab_id == slab_id ){
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
            self.peer_refs.write().expect("peer_refs.write()").push(slabref.clone());
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
}

impl Drop for SlabAgent {
    fn drop(&mut self) {
        self.net.deregister_local_slab(self.id);
    }
}