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
                            memoref.update_peer(origin_slabref, PeeringStatus::Resident);
                        }

                        should_process = self.net.apply_root_index_seed( &presence, root_index_seed );
                    }
                    &None => {
                        should_process = true;
                    }
                }

                if should_process {

                    let mentioned_slabref = SlabRef::new_from_presence( presence, &self.net );

                    if self.inject_peer_slabref( mentioned_slabref ) {
                        // TODO: should we be telling the origin slabref, or the presence slabref that we're here?
                        //       these will usually be the same, but not always

                        // Get the address that the remote slab would recogize
                        if let &Some(ref my_local_address) = origin_slabref.get_local_return_address() {
                            let my_presence = SlabPresence {
                                slab_id: my_slab.id,
                                address: my_local_address.clone(),
                                lifetime: SlabAnticipatedLifetime::Unknown
                            };

                            let my_presence_memo = Memo::new_basic(
                                my_slab.gen_memo_id(),
                                0,
                                MemoRefHead::from_memoref(memoref.clone()),
                                MemoBody::SlabPresence{ p: my_presence, r: self.get_root_index_seed() }
                            );

                            origin_slabref.send_memo( &my_ref, my_presence_memo );
                        }
                    }
                }
            }
            MemoBody::Peering(memo_id, ref presence, ref status) => {
                // Don't peer with yourself
                if presence.slab_id != my_ref.slab_id {
                    // NOTE: should never really get here, but it's possible that we got a bounced memo which we emitted in the first place
                    // TODO: Determine when this memo is superseded/stale, punt update
                    let peered_memoref = self.memorefs_by_id.entry(memo_id).or_insert_with(|| MemoRef::new_remote(memo_id));

                    peered_memoref.update_peer( &self.net.assert_slabref_from_presence( presence ), status.clone());
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
                                    MemoBody::Peering( memo.id, my_ref.presence.clone(), PeeringStatus::Participating)
                                );
                                requesting_slabref.send_memo(&my_ref, peering_memo)
                            }
                        }else{
                            let peering_memo = Memo::new(
                                my_slab.gen_memo_id(), 0,
                                MemoRefHead::from_memoref(memoref.clone()),
                                MemoBody::Peering( memo.id, my_ref.presence.clone(), PeeringStatus::NonParticipating)
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
            if let &Some(ref my_return_address) = origin_slabref.get_local_return_address() {
                let peering_memo = Memo::new(
                    my_slab.gen_memo_id(), 0,
                    MemoRefHead::from_memoref(memoref.clone()),
                    MemoBody::Peering(
                        memo.id,
                        SlabPresence {
                            slab_id:  self.id,
                            address: my_return_address.clone(),
                            lifetime: SlabAnticipatedLifetime::Unknown
                        },
                        PeeringStatus::Resident
                    )
                );
                origin_slabref.send_memo( &my_slabref, peering_memo );
            }
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
