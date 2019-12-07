use std::collections::HashMap;
//use std::collections::hash_map::Entry;

use crate::slab::{MemoId, MemoRef};
use crate::network::SlabRef;

pub struct SlabState{
    pub memorefs_by_id: HashMap<MemoId,MemoRef>,
    pub counters: SlabCounters,
    pub peer_refs: Vec<SlabRef>,
}

struct SlabCounters{
    pub last_memo_id: u32,
    pub last_subject_id: u32,
    pub memos_received: u64,
    pub memos_redundantly_received: u64,
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
        }
    }
}