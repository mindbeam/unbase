// attic is not a module

/*

//TODO: update OtherSlab to use MemoPeer?
#[derive(Debug)]
pub enum MemoOrigin<'a>{
    SameSlab,
    OtherSlab(&'a SlabRef, MemoPeeringStatus)
    // TODO: consider bifurcation into OtherSlabTrusted, OtherSlabUntrusted
    //       in cases where we want to reduce computational complexity by foregoing verification
}

*/

// TODO: convert this to reconstitute_memos ( plural )
/*
    pub fn put_memos(&self, memo_origin: &MemoOrigin, mut memos: Vec<Memo> ) -> Vec<MemoRef>{

        // TODO: Evaluate more efficient ways to group these memos by entity
        let mut entity_updates : HashMap<EntityId, Head> = HashMap::new();
        let mut memorefs = Vec::with_capacity( memos.len() );
        let mut pre_existing = 0u64;

        for memo in memos.drain(..){
            let (memoref, pre_existed) = self.memoref_from_memo_and_origin( memo, memo_origin );
            if pre_existed { pre_existing += 1 }

            self.handle_memoref( memo_origin, &memoref ); // located in memohandling.rs

            if let Some(entity_id) = memoref.entity_id {
                let mut head = entity_updates.entry( entity_id ).or_insert( Head::new() );
                head.apply_memoref(&memoref, self);
            }

            memorefs.push(memoref);
        }

        {
            let mut counters = self.counters.write().unwrap();
            counters.memos_received += memorefs.len() as u64;
            counters.memos_redundantly_received += pre_existing;
        }

        for (entity_id,head) in entity_updates {
            self.dispatch_entity_head(entity_id, &head);
        }

        memorefs
    }
*/
