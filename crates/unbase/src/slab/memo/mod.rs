/* Memo
 * A memo is an immutable message.
*/
pub mod serde;

use core::ops::Deref;
use std::{
    collections::HashMap,
    fmt,
    sync::Arc
};
use futures::future::{BoxFuture, FutureExt};

use crate::{
    memorefhead::MemoRefHead,
    network::{SlabRef,SlabPresence},
    slab::{
        EdgeSet,
        MemoPeerList,
        MemoRef,
        RelationSet,
        SlabId,
        SlabHandle,
        SubjectId,
        SubjectType,
    }
};
use crate::error::RetrieveError;
use itertools::Itertools;

//pub type MemoId = [u8; 32];
pub type MemoId = u64;

// All portions of this struct should be immutable

#[derive(Clone)]
pub struct Memo(Arc<MemoInner>);

impl Deref for Memo {
    type Target = MemoInner;
    fn deref(&self) -> &MemoInner {
        &*self.0
    }
}

pub struct MemoInner {
    pub id: u64,
    pub subject_id: Option<SubjectId>,
    pub owning_slab_id: SlabId,
    pub parents: MemoRefHead,
    pub body: MemoBody
}

#[derive(Clone, Debug)]
pub enum MemoBody{
    SlabPresence{ p: SlabPresence, r: MemoRefHead }, // TODO: split out root_index_seed conveyance to another memobody type
    Relation(RelationSet),
    Edge(EdgeSet),
    Edit(HashMap<String, String>),
    FullyMaterialized     { v: HashMap<String, String>, r: RelationSet, e: EdgeSet, t: SubjectType },
    PartiallyMaterialized { v: HashMap<String, String>, r: RelationSet, e: EdgeSet, t: SubjectType },
    Peering(MemoId,Option<SubjectId>,MemoPeerList),
    MemoRequest(Vec<MemoId>,SlabRef)
}


/*
use std::hash::{Hash, Hasher};

impl Hash for MemoId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.originSlab.hash(state);
        self.id.hash(state);
    }
}
*/

impl fmt::Debug for Memo{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Memo")
           .field("id", &self.id)
           .field("subject_id", &self.subject_id)
           .field("parents", &self.parents)
           .field("body", &self.body)
           .finish()
    }
}

impl Memo {
    pub fn new (inner: MemoInner) -> Self {
        Memo(Arc::new(inner))
    }
    pub fn get_parent_head (&self) -> MemoRefHead {
        self.parents.clone()
    }
    pub fn get_values (&self) -> Option<(HashMap<String, String>,bool)> {

        match self.body {
            MemoBody::Edit(ref v) => {
                Some((v.clone(),false))
            },
            MemoBody::FullyMaterialized { ref v, .. } => {
                Some((v.clone(),true))
            },
            _   => None
        }
    }
    pub fn get_relations (&self) -> Option<(RelationSet,bool)> {

        match self.body {
            MemoBody::Relation(ref r) => {
                Some((r.clone(), false))
            },
            MemoBody::FullyMaterialized { ref r, .. } => {
                Some((r.clone(), true))
            },
            _   => None
        }
    }
    pub fn get_edges (&self) -> Option<(EdgeSet,bool)> {

        match self.body {
            MemoBody::Edge(ref e) => {
                Some((e.clone(),false))
            },
            MemoBody::FullyMaterialized { ref e, .. }  => {
                Some((e.clone(),true))
            },
            _   => None
        }
    }
    pub fn does_peering (&self) -> bool {
        match self.body {
            MemoBody::MemoRequest(_,_) => {
                false
            }
            MemoBody::Peering(_,_,_) => {
                false
            }
            MemoBody::SlabPresence{p:_, r:_} => {
                false
            }
            _ => {
                true
            }
        }
    }
    #[tracing::instrument]
    pub fn descends<'a>  (&'a self, memoref: &'a MemoRef, slab: &'a SlabHandle) -> BoxFuture<'a, Result<bool,RetrieveError>> {
        // Not really sure if this is right

        //TODO: parallelize this
        //TODO: Use sparse-vector/beacon to avoid having to trace out the whole lineage
        //      Should be able to stop traversal once happens-before=true. Cannot descend a thing that happens after

        async move {
            // breadth-first
            for parent in self.parents.iter() {
                if parent == memoref {
                    return Ok(true)
                };
            }
            // Ok now depth
            for parent in self.parents.iter() {
                if parent.descends(&memoref, slab).await? {
                    return Ok(true)
                }
            }
            return Ok(false)
        }.boxed()
    }
}

impl MemoBody {
    pub fn summary( &self ) -> String {
        use MemoBody::*;

        match self {
            SlabPresence{ ref p, ref r } =>{
                if r.is_some() {
                    format!("SlabPresence({} at {})*", p.slab_id, p.address.to_string())
                }else{
                    format!("SlabPresence({} at {})", p.slab_id, p.address.to_string())
                }
            }
            Relation(ref rel_set) => {
                format!("RelationSet({})", rel_set.to_string() )
            },
            Edge(ref edge_set) => {
                format!("EdgeSet")
            },
            Edit(ref e) => {
                format!("Edit")
            },
            FullyMaterialized{ ref v,  ref r, ref e, ref t }  => {
                format!("FullyMaterialized")
            },
            PartiallyMaterialized{ ref v, ref r, ref e, ref t }  => {
                format!("PartiallyMaterialized")
            },
            Peering( ref memo_id, ref subject_id, ref peerlist ) =>{
                format!("Peering")
            }
            MemoRequest( ref memo_ids, ref slabref ) =>{
                format!("MemoRequest({} to {})", memo_ids.iter().join(","), slabref.slab_id)
            }
        }
    }
}
