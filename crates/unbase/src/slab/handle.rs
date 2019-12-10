use futures::channel::mpsc;
use std::sync::Arc;

use crate::network::SlabRef;
use crate::slab::{SlabPresence, SlabId, MemoId, MemoPeerList, Memo, MemoRef};
use crate::subject::SubjectId;
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

}