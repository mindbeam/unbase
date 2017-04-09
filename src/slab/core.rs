
use super::*;

impl Slab {
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

        let (memoref, _had_memoref) = self.assert_memoref(memo.id, memo.subject_id, MemoPeerList(Vec::new()), Some(memo) );
        self.consider_emit_memo(&memoref);

        memoref
    }
    pub fn reconstitute_memo ( &self, memo_id: MemoId, subject_id: Option<SubjectId>, parents: MemoRefHead, body: MemoBody, origin_slabref: &SlabRef, origin_peering_status: &MemoPeeringStatus ) -> (Memo,MemoRef,bool){
        // TODO: should probably accept a peer list, rather than generating it ourselves

        let peerlist = MemoPeerList(vec![MemoPeer{
            slabref: origin_slabref.clone(),
            status: origin_peering_status.clone()
        }]);

        let memo = Memo::new(MemoInner {
            id:             memo_id,
            owning_slab_id: self.id,
            subject_id:     subject_id,
            parents:        parents,
            body:           body
        });


        let (memoref, had_memoref) = self.assert_memoref(memo.id, memo.subject_id, peerlist, Some(memo.clone()) );

        {
            let mut counters = self.counters.write().unwrap();
            counters.memos_received += 1;
            if had_memoref {
                counters.memos_redundantly_received += 1;
            }
        }

        self.consider_emit_memo(&memoref);

        if let Some(ref memo) = memoref.get_memo_if_resident() {
            self.check_memo_waiters(memo);
            self.handle_memo_from_other_slab( memo, &memoref, &origin_slabref, origin_peering_status);
            self.do_peering(&memoref, &origin_slabref);

        }
        memoref.update_peer(origin_slabref, origin_peering_status.clone());

        if let Some(subject_id) = memoref.subject_id {
            self.dispatch_subject_head( subject_id, &memoref.to_head());
        }

        (memo, memoref, had_memoref)
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
    pub fn assert_memoref( &self, memo_id: MemoId, subject_id: Option<SubjectId>, peerlist: MemoPeerList, memo: Option<Memo>) -> (MemoRef, bool) {

        let had_memoref;
        let memoref = match self.memorefs_by_id.write().unwrap().entry(memo_id) {
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
                    if let MemoRefPtr::Remote = *mr.ptr.read().unwrap() {
                        *mr.ptr.write().unwrap() = MemoRefPtr::Resident(m)
                    }
                }
                mr.apply_peers( &peerlist );
                mr.clone()
            }
        };

        (memoref, had_memoref)
    }
    pub fn assert_slabref(&self, args: TransmitterArgs, presence: &[SlabPresence] ) -> SlabRef {
        let slab_id = args.get_slab_id();
        if let Some(slabref) = self.peer_refs.read().unwrap().iter().find(|r| r.0.slab_id == slab_id ) {
            for p in presence.iter(){
                if slabref.apply_presence(p) {
                    let new_trans = self.net.get_transmitter( &args ).expect("new_from_slab net.get_transmitter");
                    let return_address = self.net.get_return_address( &p.address ).expect("return address not found");

                    *slabref.0.tx.lock().unwrap() = new_trans;
                    *slabref.0.return_address.write().unwrap() = return_address;
                }
            }
            return slabref.clone();
        }else{
            let tx = self.net.get_transmitter( &args ).expect("new_from_slab net.get_transmitter");
            //  pick one of the presences to use as our return address
            let return_address = self.net.get_return_address( &presence[0].address ).expect("return address not found");

            let inner = SlabRefInner {
                slab_id: slab_id,
                owning_slab_id: self.id, // for assertions only?
                presence: RwLock::new(presence.to_vec()),
                tx: Mutex::new(tx),
                return_address: RwLock::new(return_address),
            };

            let slabref = SlabRef(Arc::new(inner));
            self.peer_refs.write().unwrap().push(slabref.clone());

            return slabref;
        };

    }
}
