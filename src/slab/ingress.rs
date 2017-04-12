use super::*;

impl Slab {
    pub fn handle_memo_from_other_slab( &self, memo: &Memo, memoref: &MemoRef, origin_slabref: &SlabRef ){
        println!("Slab({}).handle_memo_from_other_slab({})", self.id, memo.id );

        match memo.body {
            // This Memo is a peering status update for another memo
            MemoBody::SlabPresence{ p: ref presence, r: ref opt_root_index_seed } => {

                    println!("Slab({}).handle_memo_from_other_slab({}) B", self.id, memo.id );
                let should_process;
                match opt_root_index_seed {
                    &Some(ref root_index_seed) => {

                            println!("Slab({}).handle_memo_from_other_slab({}) C", self.id, memo.id );
                        // HACK - this should be done inside the deserialize
                        for memoref in root_index_seed.iter() {
                            memoref.update_peer(origin_slabref, MemoPeeringStatus::Resident);
                        }

                        should_process = self.net.apply_root_index_seed( &presence, root_index_seed, &self.my_ref );
                    }
                    &None => {
                        println!("Slab({}).handle_memo_from_other_slab({}) D", self.id, memo.id );
                        should_process = true;
                    }
                }

                if should_process {

                        println!("Slab({}).handle_memo_from_other_slab({}) E", self.id, memo.id );
                    if let Ok(mentioned_slabref) = self.slabref_from_presence( presence ) {
                        // TODO: should we be telling the origin slabref, or the presence slabref that we're here?
                        //       these will usually be the same, but not always

                        println!("Slab({}).handle_memo_from_other_slab({}) F", self.id, memo.id );
                        let my_presence_memoref = self.new_memo_basic(
                            None,
                            memoref.to_head(),
                            MemoBody::SlabPresence{
                                p: self.presence_for_origin( origin_slabref ),
                                r: self.get_root_index_seed()
                            }
                        );

                        println!("Slab({}).handle_memo_from_other_slab({}) G {:?}", self.id, memo.id, origin_slabref );

                        origin_slabref.send( &self.my_ref, &my_presence_memoref );
                        println!("Slab({}).handle_memo_from_other_slab({}) H", self.id, memo.id);

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
}
