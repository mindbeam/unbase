use super::*;

impl SlabShared {
    pub fn check_memo_waiters ( &mut self, memo: &Memo) {
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
    }

    pub fn handle_memo_from_other_slab( &mut self, memo: &Memo, memoref: &MemoRef, origin_slabref: &SlabRef, my_slab: &Slab, my_ref: &SlabRef  ){

        match memo.inner.body {
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

                    if let Ok(mentioned_slabref) = self.assert_slabref_from_presence( presence ) {
                        // TODO: should we be telling the origin slabref, or the presence slabref that we're here?
                        //       these will usually be the same, but not always

                        // Get the address that the remote slab would recogize
                        let my_presence = SlabPresence {
                            slab_id: my_slab.id,
                            address: origin_slabref.get_return_address(),
                            lifetime: SlabAnticipatedLifetime::Unknown
                        };

                        let memo_id = my_slab.gen_memo_id();
                        let parents = MemoRefHead::from_memoref(memoref.clone());
                        let root_index_seed = self.get_root_index_seed();


                        let my_presence_memo = Memo::new_basic(
                            memo_id,
                            0,
                            parents,
                            MemoBody::SlabPresence{ p: my_presence, r: root_index_seed },
                            &my_slab
                        );

                        origin_slabref.send_memo( &my_ref, my_presence_memo );

                    }
                }
            }
            MemoBody::Peering(memo_id, ref presence, ref status) => {
                // Don't peer with yourself

                let peered_memoref = self.memorefs_by_id.entry(memo_id).or_insert_with(|| MemoRef::from_memo_id_remote(&my_slab,memo_id));
                for p in presence.iter().filter(|p| p.slab_id != self.id ) {
                    let slabref = self.assert_slabref_from_presence( p ).expect("assert_slabref_from_presence");
                    peered_memoref.update_peer( &slabref, status.clone());
                }
            },
            MemoBody::MemoRequest(ref desired_memo_ids, ref requesting_slabref ) => {

                if requesting_slabref.0.to_slab_id != self.id {
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
                                    MemoBody::Peering( memo.id, my_ref.get_presence(), MemoPeeringStatus::Participating),
                                    &my_slab
                                );
                                requesting_slabref.send_memo(&my_ref, peering_memo)
                            }
                        }else{
                            let peering_memo = Memo::new(
                                my_slab.gen_memo_id(), 0,
                                MemoRefHead::from_memoref(memoref.clone()),
                                MemoBody::Peering( memo.id, my_ref.get_presence(), MemoPeeringStatus::NonParticipating),
                                &my_slab
                            );
                            requesting_slabref.send_memo(&my_ref, peering_memo)
                        }
                    }
                }
            }
            _ => {}
        }
    }
    pub fn do_peering_for_memo(&self, memo: &Memo, memoref: &MemoRef, origin_slabref: &SlabRef, my_slab: &Slab, my_slabref: &SlabRef) {
        // Peering memos don't get peering memos, but Edit memos do
        // Abstracting this, because there might be more types that don't do peering
        if memo.does_peering() {
            // That we received the memo means that the sender didn't think we had it
            // Whether or not we had it already, lets tell them we have it now.
            // It's useful for them to know we have it, and it'll help them STFU

            // TODO: determine if peering memo should:
            //    A. use parents at all
            //    B. and if so, what should be should we be using them for?
            //    C. Should we be sing that to determine the peered memo instead of the payload?
            //println!("MEOW {}, {:?}", my_ref );

            let peering_memo = Memo::new(
                my_slab.gen_memo_id(), 0,
                MemoRefHead::from_memoref(memoref.clone()),
                MemoBody::Peering(
                    memo.id,
                    memoref.get_presence_for_peer(origin_slabref),
                    MemoPeeringStatus::Resident
                ),
                &my_slab
            );
            origin_slabref.send_memo( &my_slabref, peering_memo );
        }

    }

    pub fn emit_memos(&self, memorefs: &Vec<MemoRef>) {
        // Emit memos for durability and notification purposes
        // At present, some memos like peering and slab presence are emitted manually.
        // TODO: This will almost certainly have to change once gossip/plumtree functionality is added

        // TODO: test each memo for durability_score and emit accordingly
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
}
