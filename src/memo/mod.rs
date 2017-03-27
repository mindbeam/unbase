/* Memo
 * A memo is an immutable message.
*/
pub mod serde;

use std::collections::HashMap;
use std::{fmt};
use std::sync::Arc;

use subject::{SubjectId};
use memoref::*;
use memorefhead::*;
use network::{SlabRef,SlabPresence};
use slab::Slab;

//pub type MemoId = [u8; 32];
pub type MemoId = u64;


#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum PeeringStatus{
    Resident,
    Participating,
    NonParticipating,
    Unknown
}

#[derive(Debug,Serialize,PartialEq)]
pub enum MemoBody{
    SlabPresence{ p: SlabPresence, r: Option<MemoRefHead> }, // TODO: split out root_index_seed conveyance to another memobody type
    Relation(HashMap<RelationSlotId,(SubjectId,MemoRefHead)>),
    Edit(HashMap<String, String>),
    FullyMaterialized     { v: HashMap<String, String>, r: HashMap<RelationSlotId,(SubjectId,MemoRefHead)> },
    PartiallyMaterialized { v: HashMap<String, String>, r: HashMap<RelationSlotId,(SubjectId,MemoRefHead)> },
    Peering(MemoId,SlabPresence,PeeringStatus),
    MemoRequest(Vec<MemoId>,SlabRef)
}

// All portions of this struct should be immutable

#[derive(Clone,PartialEq)]
pub struct Memo {
    pub id: u64,
    pub subject_id: u64,
    pub inner: Arc<MemoInner>
}
#[derive(PartialEq)]
pub struct MemoInner {
    pub id: u64,
    pub subject_id: u64,
    pub parents: MemoRefHead,
    pub body: MemoBody
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
        let inner = &self.inner;
        fmt.debug_struct("Memo")
           .field("id", &inner.id)
           .field("subject_id", &inner.subject_id)
           .field("parents", &inner.parents)
           .field("body", &inner.body)
           .finish()
    }
}

impl Memo {
    pub fn new (id: MemoId, subject_id: SubjectId, parents: MemoRefHead, body: MemoBody) -> Memo {

        println!("# Memo.new(id: {},subject_id: {}, parents: {:?}, body: {:?})", id, subject_id, parents.memo_ids(), body );

        let me = Memo {
            id:    id,
            subject_id: subject_id,
            inner: Arc::new(MemoInner {
                id:    id,
                subject_id: subject_id,
                parents: parents,
                body: body
            })
        };

        //println!("# New Memo: {:?}", me.inner.id );
        me
    }
    pub fn new_basic (id: MemoId, subject_id: SubjectId, parents: MemoRefHead, body: MemoBody) -> Self {
        Self::new(id, subject_id, parents, body)
    }
    pub fn new_basic_noparent (id: MemoId, subject_id: SubjectId, body: MemoBody) -> Self {
        Self::new(id, subject_id, MemoRefHead::new(), body)
    }
    pub fn get_parent_head (&self) -> MemoRefHead {
        self.inner.parents.clone()
    }
    pub fn get_values (&self) -> Option<(HashMap<String, String>,bool)> {

        match self.inner.body {
            MemoBody::Edit(ref v)
                => Some((v.clone(),false)),
            MemoBody::FullyMaterialized { ref v, r: _ }
                => Some((v.clone(),true)),
            _   => None
        }
    }
    pub fn get_relations (&self) -> Option<(HashMap<RelationSlotId, (SubjectId, MemoRefHead)>,bool)> {

        match self.inner.body {
            MemoBody::Relation(ref r)
                => Some((r.clone(),false)),
            MemoBody::FullyMaterialized { v: _, ref r }
                => Some((r.clone(),true)),
            _   => None
        }
    }
    pub fn does_peering (&self) -> bool {
        match self.inner.body {
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
    pub fn descends (&self, memoref: &MemoRef, slab: &Slab) -> bool {
        //TODO: parallelize this
        //TODO: Use sparse-vector/beacon to avoid having to trace out the whole lineage
        //      Should be able to stop traversal once happens-before=true. Cannot descend a thing that happens after


        // breadth-first
        for parent in self.inner.parents.iter() {
            if parent == memoref {
                return true
            };
        }

        // Ok now depth
        for parent in self.inner.parents.iter() {
            if parent.descends(&memoref,slab) {
                return true
            }
        }
        return false;
    }
}
