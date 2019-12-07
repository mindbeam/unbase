//use std::sync::Weak;
use crate::network::SlabRef;
use crate::slab::{SlabPresence, SlabId, MemoId, MemoPeerList, Memo, MemoRef};
use crate::subject::SubjectId;

pub struct SlabHandle {
    pub my_ref: SlabRef
}

impl SlabHandle{
    pub fn new (my_ref: SlabRef) -> Self {
        SlabHandle{
            my_ref
        }
    }
    pub fn is_resident(&self) -> bool {
        unimplemented!()
    }
    pub fn assert_slabref(&self, slab_id: SlabId, presence: &[SlabPresence] ) -> SlabRef {
        // agent.rs
        unimplemented!()
    }
    pub fn assert_memoref( &self, memo_id: MemoId, subject_id: Option<SubjectId>, peerlist: MemoPeerList, memo: Option<Memo>) -> (MemoRef, bool) {
        // agent.rs
        unimplemented!()
    }
}