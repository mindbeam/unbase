//TODO MERGE
use std::collections::HashMap;

use futures::{
    channel::{
        oneshot,
        mpsc,
    },
};

use crate::{
    memorefhead::MemoRefHead,
    network::SlabRef,
    slab::{
        MemoId, MemoRef, Memo
    },
    subject::SubjectId,
};

pub struct SlabState{
    pub (crate) memorefs_by_id: HashMap<MemoId,MemoRef>,
    pub (crate) counters: SlabCounters,
    pub (crate) peer_refs: Vec<SlabRef>,
    pub (crate) memo_wait_channels: HashMap<MemoId,Vec<oneshot::Sender<Memo>>>,
    pub (crate) subject_subscriptions: HashMap<SubjectId, Vec<mpsc::Sender<MemoRefHead>>>,
    pub (crate) running: bool,
}

#[derive(Debug)]
pub (crate) struct SlabCounters{
    pub last_memo_id: u32,
    pub last_subject_id: u32,
    pub memos_received: u64,
    pub memos_redundantly_received: u64
}

// SlabState is forbidden from any blocking operations
// Any code here is holding a mutex lock

impl SlabState{
    pub fn new () -> Self {
        SlabState {
            memorefs_by_id: HashMap::new(),
            counters: SlabCounters {
                last_memo_id: 5000,
                last_subject_id: 9000,
                memos_received: 0,
                memos_redundantly_received: 0,
            },
            peer_refs: Vec::new(),
            memo_wait_channels: HashMap::new(),
            subject_subscriptions: HashMap::new(),
            running: true,
        }
    }
}

impl std::fmt::Debug for SlabState {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
//        use itertools::join;

        fmt.debug_struct("SlabState")
            .field("counters", &self.counters)
//            .field( "memorefs_by_id", &(self.memorefs_by_id.keys().join(",")) )
            .finish()
    }
}