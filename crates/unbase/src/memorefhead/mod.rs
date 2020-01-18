pub mod serde;
mod projection;

use crate::slab::*;
use crate::subject::*;
use crate::context::*;
use crate::error::*;

use std::{
    mem,
    fmt,
    slice,
    collections::VecDeque,
    pin::Pin,
};

use tracing::{
    debug
};

use futures::{
    FutureExt,
    Stream,
    StreamExt,
    task::Poll,
    future::{
        BoxFuture
    }
};

// MemoRefHead is a list of MemoRefs that constitute the "head" of a given causal chain
//
// This "head" is rather like a git HEAD, insofar as it is intended to contain only the youngest
// descendents of a given causal chain. It provides mechanisms for applying memorefs, or applying
// other MemoRefHeads such that the mutated list may be pruned as appropriate given the above.

//TODO: consider renaming to OwnedMemoRefHead
#[derive(Clone, PartialEq)]
pub enum MemoRefHead {
    Null,
    Subject{
        subject_id: SubjectId,
        head:       Vec<MemoRef>
    },
    Anonymous{
        head:       Vec<MemoRef>
    }
}

// TODO: consider renaming to ExternalMemoRefHead or something like that
pub struct MemoRefHeadWithProvenance {
    pub memorefhead: MemoRefHead,
    pub slabref: SlabRef,
}

impl MemoRefHead {
    pub fn new ( owning_slab: &SlabHandle ) -> Self {
        MemoRefHead{
            head: Vec::with_capacity(5),
            owning_slab_id: owning_slab.my_ref.slab_id
        }
    }
    pub fn new_from_vec ( vec: Vec<MemoRef>, owning_slab: &SlabHandle ) -> Self {
        MemoRefHead{
            head: vec,
            owning_slab_id: owning_slab.my_ref.slab_id
        }
    }
    pub fn from_memoref (memoref: MemoRef) -> Self {
        MemoRefHead {
            owning_slab_id: memoref.owning_slab_id,
            head: vec![memoref],
        }
    }
    #[tracing::instrument]
    pub async fn apply_memoref(&mut self, new: &MemoRef, slab: &SlabHandle ) -> bool {

        // Conditionally add the new memoref only if it descends any memorefs in the head
        // If so, any memorefs that it descends must be removed
        let head = match self {
            MemoRefHead::Null => {
                if let Some(subject_id) = new.subject_id {
                    *self = MemoRefHead::Subject{
                        head: vec![new.clone()],
                        subject_id
                    };
                }else{
                    *self = MemoRefHead::Anonymous{ head: vec![new.clone()] };
                }

                return Ok(true);
            },
            MemoRefHead::Anonymous{ ref mut head } => {
                head
            },
            MemoRefHead::Subject{ ref mut head, ..} => {
                head
            }
        };

        // Conditionally add the new memoref only if it descends any memorefs in the head
        // If so, any memorefs that it descends must be removed

        // Not suuuper in love with these flag names
        let mut new_is_descended = false;
        let mut new_descends  = false;

        let mut applied  = false;
        let mut replaced  = false;

        // I imagine it's more efficient to iterate in reverse, under the supposition that
        // new items are more likely to be at the end, and that's more likely to trigger
        // the cheapest case: (existing descends new)

        // TODO - make this more async friendly.
        'existing: for i in (0..head.len()).rev() {
            let mut remove = false;
            {
                let ref mut existing = head[i];
                if existing == new {
                    return false; // we already had this

                } else if existing.descends(&new,&slab).await {
                    new_is_descended = true;

                    // IMPORTANT: for the purposes of the boolean return,
                    // the new memo does not get "applied" in this case

                    // If any memo in the head already descends the newcomer,
                    // then it doesn't get applied at all punt the whole thing
                    break 'existing;

                } else if new.descends(&existing, &slab).await {
                    new_descends = true;
                    applied = true; // descends

                    if replaced {
                        remove = true;
                    }else{
                        // Lets try real hard not to remove stuff in the middle of the vec
                        // But we only get to do this trick once, because we don't want to add duplicates
                        mem::replace( existing, new.clone() );
                        replaced = true;
                    }

                }
            }

            if remove {
                // because we're descending, we know the offset of the next items won't change
                head.remove(i);
            }
        }

        if !new_descends && !new_is_descended  {
            // if the new memoref neither descends nor is descended
            // then it must be concurrent

            head.push(new.clone());
            applied = true; // The memoref was "applied" to the MemoRefHead
        }

        // This memoref was applied if it was concurrent, or descends one or more previous memos

//        if applied {
//            debug!("Was applied - {:?}", self.memo_ids());
//        }else{
//            debug!("NOT applied - {:?}", self.memo_ids());
//        }

        applied
    }
    #[tracing::instrument]
    pub async fn apply_memorefs (&mut self, new_memorefs: &Vec<MemoRef>, slab: &SlabHandle) {
        for new in new_memorefs.iter(){
            self.apply_memoref(new, slab).await;
        }
    }
    #[tracing::instrument]
    pub async fn apply (mut self, other: &MemoRefHead, slab: &SlabHandle) -> MemoRefHead {
        // TODO make this concurrent?
        for new in other.iter(){
            self.apply_memoref( new, slab ).await;
        }

        //TODO reimplement this with immutability
        self
    }
    pub fn memo_ids (&self) -> Vec<MemoId> {
        self.head.iter().map(|m| m.id).collect()
    }
    pub fn first_subject_id (&self) -> Option<SubjectId> {
        if let Some(memoref) = self.iter().next() {
            // TODO: Could stand to be much more robust here
            memoref.subject_id
        }else{
            None
        }
    }
    pub fn to_vec (&self) -> Vec<MemoRef> {
        self.head.clone()
    }
    #[tracing::instrument]
    fn to_stream_vecdeque (&self, slab: &SlabHandle ) -> VecDeque<CausalMemoStreamItem> {

//        let mut out = VecDeque::with_capacity(self.head.len());
//        for memoref in self.head.iter() {
//            let
//            out.push_back(CausalMemoStreamItem {
//                memoref: (*memoref).clone(),
//                fut: memoref.clone().get_memo( slab ).boxed(),
//                memo: None
//            });
//        };
//        out

        self.head.iter().map(|memoref| {
            //TODO - switching to an immutable internal datastructure should mitigate the need for clones here
            CausalMemoStreamItem {
//                memoref: memoref.clone(),
                fut: memoref.clone().get_memo( slab.clone() ).boxed(),
                memo: None
            }
        }).collect()
    }
    pub fn len (&self) -> usize {
        self.head.len()
    }
    pub fn iter (&self) -> slice::Iter<MemoRef> {
        self.head.iter()
    }
    #[tracing::instrument]
    pub fn causal_memo_stream(&self, slab: &SlabHandle ) -> CausalMemoStream {
        CausalMemoStream::from_head(&self, slab )
    }
    pub async fn is_fully_materialized(&self, slab: &SlabHandle ) -> bool {
        // TODO: consider doing as-you-go distance counting to the nearest materialized memo for each descendent
        //       as part of the list management. That way we won't have to incur the below computational effort.

        for memoref in self.iter(){
            if let Ok(memo) = memoref.clone().get_memo(slab.clone()).await {
                match memo.body {
                    MemoBody::FullyMaterialized { .._ } => {},
                    _                           => { return false }
                }
            }else{
                // TODO: do something more intelligent here
                panic!("failed to retrieve memo")
            }
        }

        true
    }
}

impl fmt::Debug for MemoRefHead{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MemoRefHead::Null       => {
                fmt.debug_struct("MemoRefHead::Null").finish()
            },
            MemoRefHead::Anonymous{ ref head, .. } => {
                fmt.debug_struct("MemoRefHead::Anonymous")
                    .field("memo_refs",  head )
                    //.field("memo_ids", &self.memo_ids() )
                    .finish()
            }
            MemoRefHead::Subject{ ref subject_id, ref head } => {
                fmt.debug_struct("MemoRefHead::Subject")
                    .field("subject_id", &subject_id )
                    .field("memo_refs",  head )
                    //.field("memo_ids", &self.memo_ids() )
                    .finish()
            }
        }
    }
}

struct CausalMemoStreamItem{
//    memoref: MemoRef,
    fut: BoxFuture<'static, Result<Memo,RetrieveError>>,
    memo: Option<Memo>
}

pub struct CausalMemoStream {
    queue: VecDeque<CausalMemoStreamItem>,
    slab:  SlabHandle
}

impl fmt::Debug for CausalMemoStream {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("CausalMemoIter")
            .field("remaining",&self.queue.len())
            .finish()
    }
}

/*
  Plausible Memo Structure:
          /- E -> C -\
     G ->              -> B -> A
head ^    \- F -> D -/
     Desired iterator sequence: G, E, C, F, D, B, A ( Why? )
     Consider:                  [G], [E,C], [F,D], [B], [A]
     Arguably this should not be an iterator at all, but rather a recursive function
     Going with the iterator for now in the interest of simplicity
*/
impl CausalMemoStream {
    #[tracing::instrument]
    pub fn from_head ( head: &MemoRefHead, slab: &SlabHandle) -> Self {
        if head.owning_slab_id != slab.my_ref.slab_id {
            assert!(head.owning_slab_id == slab.my_ref.slab_id, "requesting slab does not match owning slab");
        }
        CausalMemoStream {
            queue: head.to_stream_vecdeque(slab),
            slab:  (*slab).clone()
        }
    }
}

impl Stream for CausalMemoStream {
    // TODO NEXT - change this to Result<Memo>
    type Item = Memo;

    #[tracing::instrument]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Option<Self::Item>> {
        // iterate over head memos
        // Unnecessarly complex because we're not always dealing with MemoRefs
        // Arguably heads should be stored as Vec<MemoRef> instead of Vec<Memo>

        if self.queue.len() == 0 {
            return Poll::Ready(None);
        }

        let mut nextheads = Vec::new();

        for item in self.queue.iter_mut() {
            // QUESTION: Is it bad to pass our context? We have to poll all of these, but only want to be
            // woken up when the *first* of these futures is ready. We only get one shot at setting the
            // context/waker though, so I think we just have to deal with that.
            if let None = item.memo {
                match item.fut.as_mut().poll(cx) {
                    Poll::Ready(Ok(memo)) => {
                        nextheads.push(memo.get_parent_head() );
                        item.memo = Some(memo);
                    },
                    Poll::Ready(Err(_e)) => {
                        panic!("TODO: how should we handle memo retrieval errors in the causal iterator? {:?}", _e)
                    },
                    Poll::Pending => {}
                }
            }
        }

        for nexthead in nextheads {
            let mut foo = &mut nexthead.to_stream_vecdeque(&self.slab);
            self.queue.append( &mut foo );
        }

    // TODO -make this nicer
        if let None = self.queue[0].memo {
            return Poll::Pending;
        }

        return Poll::Ready(Some( self.queue.pop_front().unwrap().memo.unwrap() ))
    }
}