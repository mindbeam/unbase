use futures::channel::mpsc;
use std::sync::Arc;

use crate::network::SlabRef;
use crate::slab::{SlabPresence, SlabId, MemoId, MemoPeerList, Memo, MemoRef, MemoInner, MemoBody};
use crate::subject::SubjectId;
use crate::memorefhead::MemoRefHead;
use crate::Slab;
use crate::slab::agent::SlabAgent;


pub struct SlabHandle {
    pub my_ref: SlabRef,
    dispatch_channel: mpsc::Sender<MemoRef>,
    pub (crate) agent: Arc<SlabAgent>,
}

impl SlabHandle {
    pub fn new(slab: &Slab) -> Self {
        SlabHandle {
            my_ref: slab.my_ref.clone(),
            dispatch_channel: slab.dispatch_channel.clone(),
            agent: slab.agent.clone()
        }
    }
    pub fn is_resident(&self) -> bool {
        unimplemented!()
    }
    pub fn assert_slabref(&self, slab_id: SlabId, presence: &[SlabPresence]) -> SlabRef {
        // agent.rs
        unimplemented!()
    }
    pub fn assert_memoref(&self, memo_id: MemoId, subject_id: Option<SubjectId>, peerlist: MemoPeerList, memo: Option<Memo>) -> (MemoRef, bool) {
        // agent.rs
        unimplemented!()
    }

    pub fn request_memo(&self, memoref: &MemoRef) -> u8 {
        // agent.rs
        unimplemented!()
    }

    pub fn memo_wait_channel(&self, memo_id: MemoId) -> futures::channel::oneshot::Receiver<Memo> {
        // slab.rs
        unimplemented!()
    }
    pub fn reconstitute_memo ( &self, memo_id: MemoId, subject_id: Option<SubjectId>, parents: MemoRefHead, body: MemoBody, origin_slabref: &SlabRef, peerlist: &MemoPeerList ) -> (Memo,MemoRef,bool){
        //println!("Slab({}).reconstitute_memo({})", self.id, memo_id );
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
        //println!("Slab({}).reconstitute_memo({}) B -> {:?}", self.id, memo_id, memoref );


        self.consider_emit_memo(&memoref);

        if let Some(ref memo) = memoref.get_memo_if_resident() {

            self.check_memo_waiters(memo);
            self.handle_memo_from_other_slab(memo, &memoref, &origin_slabref);
            self.do_peering(&memoref, &origin_slabref);

        }

        if let Some(ref tx_mutex) = self.memoref_dispatch_tx_channel {
            tx_mutex.lock().unwrap().send(memoref.clone()).unwrap()
        }

        (memo, memoref, had_memoref)
    }

}