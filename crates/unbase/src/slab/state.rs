use std::collections::HashMap;

use futures::channel::{
    mpsc,
    oneshot,
};

use crate::{
    head::Head,
    network::SlabRef,
    slab::{
        EntityId,
        Memo,
        MemoId,
        MemoRef,
    },
};

/// SlabState stores all state for a slab
/// It may ONLY be owned/touched by SlabAgent. No exceptions.
/// Consider making SlabState a child of SlabAgent to further discourage this
pub(super) struct SlabState {
    pub memorefs_by_id:       HashMap<MemoId, MemoRef>,
    pub counters:             SlabCounters,
    pub peer_refs:            Vec<SlabRef>,
    pub memo_wait_channels:   HashMap<MemoId, Vec<oneshot::Sender<Memo>>>,
    pub entity_subscriptions: HashMap<EntityId, Vec<mpsc::Sender<Head>>>,
    pub index_subscriptions:  Vec<mpsc::Sender<Head>>,
    pub running:              bool,
}

#[derive(Debug)]
pub(crate) struct SlabCounters {
    pub last_memo_id:               u32,
    pub last_entity_id:             u32,
    pub memos_received:             u64,
    pub memos_redundantly_received: u64,
}

// SlabState is forbidden from any blocking operations
// Any code here is holding a mutex lock

impl SlabState {
    pub fn new() -> Self {
        SlabState { memorefs_by_id:       HashMap::new(),
                    counters:             SlabCounters { last_memo_id:               5000,
                                                         last_entity_id:             9000,
                                                         memos_received:             0,
                                                         memos_redundantly_received: 0, },
                    peer_refs:            Vec::new(),
                    memo_wait_channels:   HashMap::new(),
                    entity_subscriptions: HashMap::new(),
                    index_subscriptions:  Vec::new(),
                    running:              true, }
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
