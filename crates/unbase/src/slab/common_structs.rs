use std::ops::Deref;
use std::fmt;
use std::collections::HashMap;

use crate::network::{TransportAddress, SlabRef};
use crate::slab::SlabId;
use crate::memorefhead::{RelationSlotId, MemoRefHead};
use crate::subject::SubjectId;

/// SlabPresence represents the expected reachability of a given Slab
/// Including Transport address and anticipated lifetime
#[derive(Clone, Serialize, Deserialize)]
pub struct SlabPresence {
    pub slab_id: SlabId, // TODO replace this witl SlabRef
    pub address: TransportAddress,
    pub lifetime: SlabAnticipatedLifetime,
}
impl PartialEq for SlabPresence {
    fn eq(&self, other: &SlabPresence) -> bool {
        // When comparing equality, we can skip the anticipated lifetime
        self.slab_id == other.slab_id && self.address == other.address
    }
}
impl fmt::Debug for SlabPresence {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SlabPresence")
            .field("slab_id", &self.slab_id)
            .field("address", &self.address.to_string())
            .field("lifetime", &self.lifetime)
            .finish()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SlabAnticipatedLifetime {
    Ephmeral,
    Session,
    Long,
    VeryLong,
    Unknown,
}

#[derive(Clone,Debug)]
pub struct MemoPeerList(pub Vec<MemoPeer>);

impl MemoPeerList {
    pub fn new(list: Vec<MemoPeer>) -> Self {
        MemoPeerList(list)
    }
    pub fn clone(&self) -> Self {
        MemoPeerList(self.0.clone())
    }
    pub fn slab_ids(&self) -> Vec<SlabId> {
        self.0.iter().map(|p| p.slabref.slab_id).collect()
    }
    pub fn apply_peer(&mut self, peer: MemoPeer) -> bool {
        // assert!(self.owning_slab_id == peer.slabref.owning_slab_id, "apply_peer for dissimilar owning_slab_id peer" );

        let peerlist = &mut self.0;
        {
            if let Some(my_peer) = peerlist.iter_mut()
                .find(|p| p.slabref.slab_id == peer.slabref.slab_id) {
                if peer.status != my_peer.status {
                    // same slabref, so no need to apply the peer presence
                    my_peer.status = peer.status;
                    return true;
                } else {
                    return false;
                }
            }
        }

        peerlist.push(peer);
        true
    }
}

impl Deref for MemoPeerList {
    type Target = Vec<MemoPeer>;
    fn deref(&self) -> &Vec<MemoPeer> {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct MemoPeer {
    pub slabref: SlabRef,
    pub status: MemoPeeringStatus,
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum MemoPeeringStatus {
    Resident,
    Participating,
    NonParticipating,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct RelationSlotSubjectHead(pub HashMap<RelationSlotId, (SubjectId, MemoRefHead)>);

impl RelationSlotSubjectHead {
    pub fn empty() -> Self {
        RelationSlotSubjectHead(HashMap::new())
    }
    pub fn single(slot_id: RelationSlotId, subject_id: SubjectId, head: MemoRefHead) -> Self {
        let mut hashmap = HashMap::new();
        hashmap.insert(slot_id, (subject_id, head));
        RelationSlotSubjectHead(hashmap)
    }
}

impl Deref for RelationSlotSubjectHead {
    type Target = HashMap<RelationSlotId, (SubjectId, MemoRefHead)>;
    fn deref(&self) -> &HashMap<RelationSlotId, (SubjectId, MemoRefHead)> {
        &self.0
    }
}
