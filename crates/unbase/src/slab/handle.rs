use futures::channel::mpsc;
use std::sync::Arc;
use futures::future::{select,Either};

use crate::network::{SlabRef, TransportAddress};
use crate::slab::{SlabPresence, MemoId, MemoPeerList, Memo, MemoRef, MemoBody, SlabAnticipatedLifetime};
use crate::subject::SubjectId;
use crate::{Slab, Network};
use crate::slab::agent::SlabAgent;
use crate::context::Context;
use crate::memorefhead::MemoRefHead;
use crate::error::RetrieveError;
use timer::Delay;


#[derive(Clone)]
pub struct SlabHandle {
    pub my_ref: SlabRef,
    pub (crate) net: Network,
    dispatch_channel: mpsc::Sender<MemoRef>,
    pub (crate) agent: Arc<SlabAgent>,
}

impl SlabHandle {
    pub fn new(slab: &Slab) -> Self {
        SlabHandle {
            my_ref: slab.my_ref.clone(),
            net: slab.net.clone(),
            dispatch_channel: slab.dispatch_channel.clone(),
            agent: slab.agent.clone()
        }
    }
    pub fn is_resident(&self) -> bool {
        unimplemented!()
    }
    pub fn assert_memoref(&self, memo_id: MemoId, subject_id: Option<SubjectId>, peerlist: MemoPeerList, memo: Option<Memo>) -> (MemoRef, bool) {
        // agent.rs
        unimplemented!()
    }

    pub async fn request_memo(&self, memoref: &MemoRef) -> Result<Memo,RetrieveError> {

        // we're looking for this memo
        let channel = self.agent.memo_wait_channel(memoref.id);

        // send the request
        let request_memo = self.agent.new_memo_basic(
            None,
            MemoRefHead::new(), // TODO: how should this be parented?
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

            let timeout = Delay::new( duration );
            match select(channel, timeout).await {
                Either::Left((Ok(memo),_)) => {
                    return Ok(memo)
                },
                _ => {
                    // timed out or canceled
                }
            }
        }

        Err(RetrieveError::NotFoundByDeadline)
    }
    pub fn generate_subject_id(&self) -> SubjectId {
        self.agent.generate_subject_id()
    }
    pub fn subscribe_subject (&self, subject_id: u64, context: &Context) {
        self.agent.subscribe_subject(subject_id, context);
    }
    pub fn unsubscribe_subject (&self,  subject_id: u64, context: &Context ){
        self.agent.unsubscribe_subject(subject_id, context);
    }
    pub fn slabref_from_local_slab(&self, peer_slab: &SlabHandle) -> SlabRef {

        //let args = TransmitterArgs::Local(&peer_slab);
        let presence = SlabPresence{
            slab_id: peer_slab.my_ref.slab_id,
            address: TransportAddress::Local,
            lifetime: SlabAnticipatedLifetime::Unknown
        };

        self.agent.assert_slabref(peer_slab.my_ref.slab_id, &vec![presence])
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