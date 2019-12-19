use futures::channel::mpsc;
use std::sync::Arc;
use futures::future::{select,Either};

use crate::network::{SlabRef, TransportAddress};
use crate::slab::{SlabPresence, Memo, MemoRef, MemoBody, SlabAnticipatedLifetime, MemoId};
use crate::subject::SubjectId;
use crate::Network;
use crate::slab::agent::SlabAgent;
use crate::context::Context;
use crate::memorefhead::MemoRefHead;
use crate::error::RetrieveError;
use timer::Delay;


#[derive(Clone)]
pub struct SlabHandle {
    pub (crate) my_ref: SlabRef,
    pub (crate) net: Network,
    pub (crate) dispatch_channel: mpsc::Sender<MemoRef>,
    pub (crate) agent: Arc<SlabAgent>,
}

impl SlabHandle {
    pub fn is_running(&self) -> bool {
        self.agent.is_running()
    }
//    pub fn assert_memoref(&self, memo_id: MemoId, subject_id: Option<SubjectId>, peerlist: MemoPeerList, memo: Option<Memo>) -> (MemoRef, bool) {
//        // agent.rs
//        unimplemented!()
//    }

    pub async fn request_memo(&self, memoref: &MemoRef) -> Result<Memo, RetrieveError> {

        // we're looking for this memo
        let mut channel = self.agent.memo_wait_channel(memoref.id);

        // send the request
        let request_memo = self.new_memo_basic(
            None,
            MemoRefHead::new( self ), // TODO: how should this be parented?
            MemoBody::MemoRequest(
                vec![memoref.id],
                self.my_ref.clone()
            )
        );

        use std::time;
        let duration = time::Duration::from_millis(1000);

        for _ in 0..3 {
            // TODO: move this to new_memo
            let mut sent = 0u8;
            {
                for peer in memoref.peerlist.read().unwrap().iter().take(5) {
                    peer.slabref.send(&self.my_ref, &request_memo.clone());
                    sent += 1;
                }
            }

            if sent == 0 {
                return Err(RetrieveError::NotFound)
            }

            let timeout = Delay::new(duration);
            match select(channel, timeout).await {
                Either::Left((Ok(memo), _)) => {
                    return Ok(memo)
                },
                Either::Left((Err(_canceled), _)) => {
                    // the channel was canceled by the sender
                    return Err(RetrieveError::NotFound);
                },
                Either::Right((_, ch)) => {
                    // timed out. Preserve the memo wait channel
                    channel = ch;
                }
            }
        }

        Err(RetrieveError::NotFoundByDeadline)
    }
    pub fn new_memo_basic (&self, subject_id: Option<SubjectId>, parents: MemoRefHead, body: MemoBody) -> MemoRef {
        self.agent.new_memo(subject_id, parents, body)
    }
    pub fn new_memo_basic_noparent (&self, subject_id: Option<SubjectId>, body: MemoBody) -> MemoRef {
        self.agent.new_memo(subject_id, MemoRefHead::new(self), body)
    }
    pub fn generate_subject_id(&self) -> SubjectId {
        self.agent.generate_subject_id()
    }
    pub fn subscribe_subject(&self, subject_id: u64, context: &Context) {
        self.agent.subscribe_subject(subject_id, context);
    }
    pub fn unsubscribe_subject(&self, subject_id: u64, context: &Context) {
        self.agent.unsubscribe_subject(subject_id, context);
    }
    pub fn slabref_from_local_slab(&self, peer_slab: &SlabHandle) -> SlabRef {

        //let args = TransmitterArgs::Local(&peer_slab);
        let presence = SlabPresence {
            slab_id: peer_slab.my_ref.slab_id,
            address: TransportAddress::Local,
            lifetime: SlabAnticipatedLifetime::Unknown
        };

        self.agent.assert_slabref(peer_slab.my_ref.slab_id, &vec![presence])
    }
    pub fn remotize_memo_ids(&self, memo_ids: &[MemoId]) -> Result<(), String> {
        self.agent.remotize_memo_ids(memo_ids)
    }
    pub fn peer_slab_count (&self) -> usize {
        self.agent.peer_slab_count()
    }
    pub fn count_of_memorefs_resident( &self ) -> u32 {
        self.agent.count_of_memorefs_resident()
    }
    pub fn count_of_memos_received( &self ) -> u64 {
        self.agent.count_of_memos_received()
    }
    pub fn count_of_memos_reduntantly_received( &self ) -> u64 {
        self.agent.count_of_memos_reduntantly_received()
    }
}

impl std::fmt::Debug for SlabHandle {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("SlabHandle")
            .field("slab_id", &self.my_ref.slab_id)
            .field("agent", &self.agent)
            .finish()
    }
}