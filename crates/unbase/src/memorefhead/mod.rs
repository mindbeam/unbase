pub mod serde;

use crate::{
    error::{
        RetrieveError,
        WriteError,
    },
    slab::{
        SubjectType,
        MAX_SLOTS,
        MemoRef,
        MemoBody,
        Memo,
        MemoId,
        SlabId,
        SlabHandle,
        SlabRef,
        SubjectId,
        RelationSlotId,
        EdgeSet,
        EdgeLink,
        RelationSet,
    },
};

use std::{
    mem,
    fmt,
    slice,
    collections::{
        HashMap,
        VecDeque,
    },
    pin::Pin,
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

use tracing::debug;

// MemoRefHead is a list of MemoRefs that constitute the "head" of a given causal chain
//
// This "head" is rather like a git HEAD, insofar as it is intended to contain only the youngest
// descendents of a given causal chain. It provides mechanisms for applying memorefs, or applying
// other MemoRefHeads such that the mutated list may be pruned as appropriate given the above.

//TODO: consider renaming to OwnedMemoRefHead
#[derive(Clone, PartialEq)]

// TODO - consider changing this to a linkedlist instead of a Vec, because MOST of the time it's going to be a single memoref
// This will allow us to save allocations, and potentially have a number of traversal operations happen entirely on the stack?
//struct Link {
//    memoref: MemoRef,
//    //90+% of the time this will be None
//    next: Option<Box<Link>>
//}

pub enum MemoRefHead {
    Null,
    Subject{
        owning_slab_id: SlabId,
        subject_id: SubjectId,
        head:       Vec<MemoRef>
    },
    Anonymous{
        owning_slab_id: SlabId,
        head:       Vec<MemoRef>
    }
}

// TODO: consider renaming to ExternalMemoRefHead or something like that
pub struct MemoRefHeadWithProvenance {
    pub memorefhead: MemoRefHead,
    pub slabref: SlabRef,
}

/// MemoRefHead takes &SlabHandle on all calls, because it is an agent of storage and referentiality, NOT an enforcer of consistency
impl MemoRefHead {
//    pub fn new_record( slab: &SlabHandle ){
//
//    }
    pub fn new_index ( slab: &SlabHandle, values: HashMap<String,String> ) -> MemoRefHead {
        let id = slab.generate_subject_id(SubjectType::IndexNode);

        slab.new_memo(
            Some(id),
            MemoRefHead::Null,
            MemoBody::FullyMaterialized {v: values, r: RelationSet::empty(), e: EdgeSet::empty(), t: SubjectType::IndexNode }
        ).to_head()
    }
    #[tracing::instrument]
    pub async fn mut_apply_memoref(&mut self, new: &MemoRef, slab: &SlabHandle ) -> Result<bool,WriteError> {

        // Conditionally add the new memoref only if it descends any memorefs in the head
        // If so, any memorefs that it descends must be removed
        let head = match self {
            MemoRefHead::Null => {
                if let Some(subject_id) = new.subject_id {
                    *self = MemoRefHead::Subject{
                        owning_slab_id: new.owning_slab_id,
                        head: vec![new.clone()],
                        subject_id
                    };
                }else{
                    *self = MemoRefHead::Anonymous{
                        owning_slab_id: new.owning_slab_id,
                        head: vec![new.clone()]
                    };
                }

                return Ok(true);
            },
            MemoRefHead::Anonymous{ ref mut head, .. } => {
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
                    return Ok(false); // we already had this

                } else if existing.descends(&new,&slab).await? {
                    new_is_descended = true;

                    // IMPORTANT: for the purposes of the boolean return,
                    // the new memo does not get "applied" in this case

                    // If any memo in the head already descends the newcomer,
                    // then it doesn't get applied at all punt the whole thing
                    break 'existing;

                } else if new.descends(&existing, &slab).await? {
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

        Ok(applied)
    }
    #[tracing::instrument]
    pub async fn mut_apply_memorefs(&mut self, new_memorefs: &Vec<MemoRef>, slab: &SlabHandle) -> Result<bool,WriteError> {
        let mut did_apply = false;
        
        for new in new_memorefs.iter(){
            if self.mut_apply_memoref(new, slab).await? {
                did_apply = true;
            }
        }

        Ok(did_apply)
    }
    #[tracing::instrument]
    pub async fn mut_apply(&mut self, other: &MemoRefHead, slab: &SlabHandle) -> Result<bool,WriteError> {
        match other {
            MemoRefHead::Null => {
                Ok(false)
            },
            MemoRefHead::Anonymous{ ref head, .. }   => {
                let mut applied = false;
                for new in head.iter() {
                    if self.mut_apply_memoref(new, slab ).await? {
                        applied = true;
                    };
                }
                Ok(applied)
            },
            MemoRefHead::Subject{ ref head, .. } => {
                let mut applied = false;
                for new in head.iter() {
                    if self.mut_apply_memoref(new, slab ).await? {
                        applied = true;
                    };
                }
                Ok(applied)
            }
        }
    }
    /// Immutably apply a second head to this one
    #[tracing::instrument]
    pub async fn apply(&self, other: &MemoRefHead, slab: &SlabHandle) -> Result<(MemoRefHead, bool),WriteError> {
        let mut applied = false;
        // This is just a temporary API hack so we don't forget to make this nicer later with internal _immutability_.
        //TODO reimplement this with immutabilityZ

        let mut hack_self = self.clone();
        for new in other.iter(){
            if hack_self.mut_apply_memoref(new, slab ).await? {
                applied = true;
            };
        }

        Ok((hack_self, applied))
    }
    pub async fn descends_or_contains (&self, other: &MemoRefHead, slab: &SlabHandle) -> Result<bool,RetrieveError> {

        // there's probably a more efficient way to do this than iterating over the cartesian product
        // we can get away with it for now though I think
        // TODO: revisit when beacons are implemented
        match *self {
            MemoRefHead::Null             => Ok(false),
            MemoRefHead::Subject{ ref head, .. } | MemoRefHead::Anonymous{ ref head, .. } => {
                match *other {
                    MemoRefHead::Null             => Ok(false),
                    MemoRefHead::Subject{ head: ref other_head, .. } | MemoRefHead::Anonymous{ head: ref other_head, .. } => {
                        if head.len() == 0 || other_head.len() == 0 {
                            return Ok(false) // searching for positive descendency, not merely non-ascendency
                        }
                        for memoref in head.iter(){
                            for other_memoref in other_head.iter(){
                                if memoref == other_memoref {
                                    //
                                } else if !memoref.descends(other_memoref, slab).await? {
                                    return Ok(false);
                                }
                            }
                        }

                        Ok(true)
                    }
                }
            }
        }
    }
    pub fn memo_ids (&self) -> Vec<MemoId> {
        match *self {
            MemoRefHead::Null => Vec::new(),
            MemoRefHead::Subject{ ref head, .. } | MemoRefHead::Anonymous{ ref head, .. } => head.iter().map(|m| m.id).collect()
        }
    }
    pub fn subject_id (&self) -> Option<SubjectId> {
        match *self {
            MemoRefHead::Null | MemoRefHead::Anonymous{..} => None,
            MemoRefHead::Subject{ subject_id, .. }     => Some(subject_id)
        }
    }
    pub fn owning_slab_id (&self) -> Option<SlabId> {
        match *self {
            MemoRefHead::Null => None,
            MemoRefHead::Anonymous { owning_slab_id, .. } => Some(owning_slab_id),
            MemoRefHead::Subject{ owning_slab_id, .. }   => Some(owning_slab_id),
        }
    }
    pub fn is_some (&self) -> bool {
        match *self {
            MemoRefHead::Null => false,
            _                 => true
        }
    }
    pub fn to_vec (&self) -> Vec<MemoRef> {
        match *self {
            MemoRefHead::Null => vec![],
            MemoRefHead::Anonymous { ref head, .. } => head.clone(),
            MemoRefHead::Subject{  ref head, .. }   => head.clone()
        }
    }
    pub fn to_vecdeque (&self) -> VecDeque<MemoRef> {
        match *self {
            MemoRefHead::Null       => VecDeque::new(),
            MemoRefHead::Anonymous { ref head, .. } => VecDeque::from(head.clone()),
            MemoRefHead::Subject{  ref head, .. }   => VecDeque::from(head.clone())
        }
    }
    pub fn len (&self) -> usize {
        match *self {
            MemoRefHead::Null       =>  0,
            MemoRefHead::Anonymous { ref head, .. } => head.len(),
            MemoRefHead::Subject{  ref head, .. }   => head.len()
        }
    }
    pub fn iter (&self) -> slice::Iter<MemoRef> {

        // This feels pretty stupid. Probably means something is wrong with the factorization of MRH
        static EMPTY : &'static [MemoRef] = &[];

        match *self {
            MemoRefHead::Null                    => EMPTY.iter(), // HACK
            MemoRefHead::Anonymous{ ref head, .. }   => head.iter(),
            MemoRefHead::Subject{ ref head, .. } => head.iter()
        }
    }
    #[tracing::instrument]
    fn to_stream_vecdeque (&self, slab: &SlabHandle ) -> VecDeque<CausalMemoStreamItem> {

        let head = match self {
            MemoRefHead::Null                                       =>  return VecDeque::new(),
            MemoRefHead::Anonymous { ref head, .. } => head,
            MemoRefHead::Subject{  ref head, .. }   => head
        };

        head.iter().map(|memoref| {
            //TODO - switching to an immutable internal datastructure should mitigate the need for clones here
            CausalMemoStreamItem {
                fut: memoref.clone().get_memo( slab.clone() ).boxed(),
                memo: None
            }
        }).collect()
    }
    #[tracing::instrument]
    pub fn causal_memo_stream(&self, slab: SlabHandle ) -> CausalMemoStream {
        CausalMemoStream::from_head(&self, slab )
    }
    pub async fn is_fully_materialized(&self, slab: &SlabHandle ) -> Result<bool,RetrieveError> {
        // TODO: consider doing as-you-go distance counting to the nearest materialized memo for each descendent
        //       as part of the list management. That way we won't have to incur the below computational effort.

        for memoref in self.iter(){
            let memo = memoref.clone().get_memo(slab.clone()).await?;
            match memo.body {
                MemoBody::FullyMaterialized {..} => {},
                _ =>  return Ok(false)
            }
        }

        Ok(true)
    }
    /// Notify whomever needs to know that a new subject has been created
    pub async fn get_value ( &mut self, slab: &SlabHandle, key: &str ) -> Result<Option<String>, RetrieveError> {
        //TODO: consider creating a consolidated projection routine for most/all uses
        let mut memostream = self.causal_memo_stream(slab.clone()).boxed();
        while let Some(memo) = memostream.next().await {
            //println!("# \t\\ Considering Memo {}", memo.id );
            if let Some((values, materialized)) = memo?.get_values() {
                if let Some(v) = values.get(key) {
                    return Ok(Some(v.clone()));
                }else if materialized {
                    return Ok(None); //end of the line here
                }
            }
        }

        Err(RetrieveError::MemoLineageError)
    }
    pub async fn get_relation ( &mut self, slab: &SlabHandle, key: RelationSlotId ) -> Result<Option<SubjectId>, RetrieveError> {
        //println!("# Subject({}).get_relation({})",self.id,key);

        let mut memostream = self.causal_memo_stream(slab.clone());
        while let Some(memo) = memostream.next().await {

            let memo = memo?;
            if let Some((relations,materialized)) = memo.get_relations(){
                debug!("# \t\\ Considering Memo {}, Head: {:?}, Relations: {:?}", memo.id, memo.get_parent_head(), relations );
                if let Some(maybe_subject_id) = relations.get(&key) {
                    return match *maybe_subject_id {
                        Some(subject_id) => Ok(Some(subject_id)),
                        None                 => Ok(None)
                    };
                }else if materialized {
                    debug!("\n# \t\\ Not Found (materialized)" );
                    return Ok(None);
                }
            }
        }

        debug!("Not Found" );
        Err(RetrieveError::MemoLineageError)
    }
    pub async fn get_edge(&mut self, slab: &SlabHandle, key: RelationSlotId ) -> Result<Option<MemoRefHead>, RetrieveError> {
        let mut memostream = self.causal_memo_stream(slab.clone());

        while let Some(memo) = memostream.next().await {

            let memo = memo?;
            if let Some((edges,materialized)) = memo.get_edges(){
                debug!("# \t\\ Considering Memo {}, Head: {:?}, Relations: {:?}", memo.id, memo.get_parent_head(), edges );

                if let Some(head) = edges.get(&key) {
                    // TODO POSTMERGE this is likely buggy - shouldn't we be looking at all of the memorefs in the head in case of concurrencies?

                    return Ok(Some(head.clone()));
                }else if materialized {
                    debug!("\n# \t\\ Not Found (materialized)" );
                    return Ok(None);
                }
            }
        }

        debug!("Not Found" );
        Err(RetrieveError::MemoLineageError)
    }
    pub async fn set_value (&mut self, slab: &SlabHandle, key: &str, value: &str) -> Result<(),WriteError> {
        let mut vals = HashMap::new();
        vals.insert(key.to_string(), value.to_string());

        let subject_id = self.subject_id();

        // TODO - do this in a single swap? (fairly certain that requires unsafe)
        let mut head = MemoRefHead::Null;
        std::mem::swap(self, &mut head);

        let mut new_head = slab.new_memo(
            subject_id,
            head,
            MemoBody::Edit(vals)
        ).to_head();

        std::mem::swap(self, &mut new_head);

        // We shouldn't need to apply the new memoref. It IS the new head
        // self.apply_memoref(&memoref, &slab).await?;

        Ok(())
    }
    pub async fn set_relation (&mut self, slab: &SlabHandle, key: RelationSlotId, relation: &Self) -> Result<(),WriteError> {
        //println!("# Subject({}).set_relation({}, {})", &self.id, key, relation.id);
        let mut relationset = RelationSet::empty();

        let subject_id = relation.subject_id().ok_or( WriteError::BadTarget )?;

        relationset.insert( key, subject_id );

        let subject_id = self.subject_id();

        // TODO - do this in a single swap? May require unsafe
        let mut head = MemoRefHead::Null;
        std::mem::swap(self, &mut head,);

        let mut new_head = slab.new_memo(
            subject_id,
            head,
            MemoBody::Relation(relationset)
        ).to_head();

        std::mem::swap(self, &mut new_head);

        // We shouldn't need to apply the new memoref. It IS the new head
        // self.apply_memoref(&memoref, &slab).await?;

        Ok(())
    }
    pub fn set_edge (&mut self, slab: &SlabHandle, key: RelationSlotId, target: MemoRefHead ) {
        debug!("# Subject({:?}).set_edge({}, {:?})", &self.subject_id(), key, target.subject_id() );

        let mut edgeset = EdgeSet::empty();
        edgeset.insert(key, target);

        let subject_id = self.subject_id();

        // TODO - do this in a single swap? May require unsafe
        let mut parents = MemoRefHead::Null;
        std::mem::swap(self, &mut parents);

        let mut new_head = slab.new_memo(
            subject_id,
            parents,
            MemoBody::Edge(edgeset)
        ).to_head();

        std::mem::swap(self, &mut new_head);

        // We shouldn't need to apply the new memoref. It IS the new head
        // self.apply_memoref(&memoref, &slab).await?;
    }

    #[tracing::instrument]
    pub async fn get_all_memo_ids ( &self, slab: SlabHandle ) -> Result<Vec<MemoId>,RetrieveError> {
        let mut memostream = self.causal_memo_stream( slab );

        let mut memo_ids = Vec::new();
        while let Some(memo) = memostream.next().await {
            memo_ids.push(memo?.id);
        }
        Ok(memo_ids)
    }
//    pub fn fully_materialize( &self, slab: &Slab ) {
//        // TODO: consider doing as-you-go distance counting to the nearest materialized memo for each descendent
//        //       as part of the list management. That way we won't have to incur the below computational effort.
//    }

    // Kind of a brute force way to do this
    // TODO: Consider calculating deltas during memoref application,
    //       and use that to perform a minimum cost subject_head_link edit

    // TODO: This projection method is probably wrong, as it does not consider how to handle concurrent edge-setting
    //       this problem applies to causal_memo_stream itself really, insofar as it should return sets of concurrent memos to be merged rather than individual memos
    // This in turn raises questions about how relations should be merged

    /// Project all edge links based only on the causal history of this head.
    /// The name is pretty gnarly, and this is very ripe for refactoring, but at least it says what it does.
    pub async fn project_all_edge_links_including_empties (&self, slab: &SlabHandle) -> Result<Vec<EdgeLink>,RetrieveError> {
        let mut edge_links : Vec<Option<EdgeLink>> = Vec::with_capacity(MAX_SLOTS);

        // None is an indication that we've not yet visited this slot, and that it is thus eligible for setting
        for _ in 0..MAX_SLOTS as usize {
            edge_links.push(None);
        }

        let mut memostream = self.causal_memo_stream(slab.clone());
        while let Some(memo) = memostream.next().await {

            match memo?.body {
                MemoBody::FullyMaterialized { e : ref edgeset, .. } => {

                    // Iterate over all the entries in this EdgeSet
                    for (slot_id,rel_head) in &edgeset.0 {

                        // Only consider the non-visited slots
                        if let None = edge_links[ *slot_id as usize ] {
                            edge_links[ *slot_id as usize ] = Some(match *rel_head {
                                MemoRefHead::Null  => EdgeLink::Vacant{ slot_id: *slot_id },
                                _                  => EdgeLink::Occupied{ slot_id: *slot_id, head: rel_head.clone() }
                            });
                        }
                    }

                    break;
                    // Fully Materialized memo means we're done here
                },
                MemoBody::Edge(ref r) => {
                    for (slot_id,rel_head) in r.iter() {

                        // Only consider the non-visited slots
                        if let None = edge_links[ *slot_id as usize ] {
                            edge_links[ *slot_id as usize ] = Some(
                                match *rel_head {
                                    MemoRefHead::Null  => EdgeLink::Vacant{ slot_id: *slot_id },
                                    _                  => EdgeLink::Occupied{ slot_id: *slot_id, head: rel_head.clone() }
                                }
                            )
                        }
                    }
                },
                _ => {}
            }
        }

        let out : Vec<EdgeLink> = edge_links.iter().enumerate().map(|(slot_id,maybe_link)| {
            // Fill in the non-visited links with vacants
            match maybe_link {
                None           => EdgeLink::Vacant{ slot_id: slot_id as RelationSlotId },
                Some(ref link) => link.clone()
            }
        }).collect();

        Ok(out)
    }
    /// Contextualized projection of occupied edges
    pub async fn project_occupied_edges (&self, slab: &SlabHandle) -> Result<Vec<EdgeLink>,RetrieveError> {
        let mut visited = [false; MAX_SLOTS];
        let mut edge_links : Vec<EdgeLink> = Vec::new();

        let mut memostream = self.causal_memo_stream(slab.clone());
        'memo: while let Some(memo) = memostream.next().await {

            let memo = memo?;

            let (edgeset,last) = match memo.body {
                MemoBody::FullyMaterialized { e : ref edgeset, .. } => {
                    (edgeset,true)
                },
                MemoBody::Edge(ref edgeset) => {
                    (edgeset,false)
                },
                _ => continue 'memo
            };

            for (slot_id,rel_head) in edgeset.iter() {
                // Only consider the non-visited slots
                if !visited[ *slot_id as usize] {
                    visited[ *slot_id as usize] = true;

                    match *rel_head {
                        MemoRefHead::Subject{..} | MemoRefHead::Anonymous{..} => {
                            edge_links.push( EdgeLink::Occupied{ slot_id: *slot_id, head: rel_head.clone() });
                        },
                        MemoRefHead::Null => {}
                    };
                }
            }

            if last {
                break;
            }
        }

        Ok(edge_links)
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
            MemoRefHead::Subject{ ref subject_id, ref head, .. } => {
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
    pub fn from_head ( head: &MemoRefHead, slab: SlabHandle) -> Self {
        match head.owning_slab_id() {
            Some(id) if id != slab.my_ref.slab_id => {
                panic!("requesting slab does not match owning slab");
            },
            _ => {}
        }

        CausalMemoStream {
            queue: head.to_stream_vecdeque(&slab),
            slab:  slab
        }
    }
}

impl Stream for CausalMemoStream {
    type Item = Result<Memo,RetrieveError>;

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

        return Poll::Ready(Some(Ok( self.queue.pop_front().unwrap().memo.unwrap() )))
    }
}