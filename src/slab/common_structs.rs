use super::*;
use network::TransportAddress;

/// SlabPresence represents the expected reachability of a given Slab
/// Including Transport address and anticipated lifetime
#[derive(Clone, Deserialize)]
pub struct SlabPresence{
    pub slab_id: SlabId,
    pub address: TransportAddress,
    pub lifetime: SlabAnticipatedLifetime
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
            .field("address", &self.address.to_string() )
            .field("lifetime", &self.lifetime)
            .finish()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SlabAnticipatedLifetime{
    Ephmeral,
    Session,
    Long,
    VeryLong,
    Unknown
}

#[derive(Clone,Debug)]
pub struct MemoPeerList (pub Vec<MemoPeer>);

impl MemoPeerList {
    pub fn new(list: Vec<MemoPeer>) -> Self {
        MemoPeerList(list)
    }
    pub fn clone(&self) -> Self {
        MemoPeerList(self.0.clone())
    }
    pub fn clone_for_slab (&self, from_slabref: &SlabRef, to_slab: &Slab) -> Self {
        MemoPeerList( self.0.iter().map(|p| {
            MemoPeer{
                slabref: p.slabref.clone_for_slab(from_slabref, to_slab),
                status: p.status.clone()
            }
        }).collect())
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
    pub status: MemoPeeringStatus
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum MemoPeeringStatus{
    Resident,
    Participating,
    NonParticipating,
    Unknown
}

#[derive(Clone, Debug)]
pub struct RelationSlotSubjectHead(pub HashMap<RelationSlotId,(SubjectId,MemoRefHead)>);

impl RelationSlotSubjectHead {
    pub fn clone_for_slab(&self, from_slabref: &SlabRef, to_slab: &Slab ) -> Self {

        let new = self.0.iter().map(|(slot_id,&(subject_id,ref mrh))| {
            (*slot_id, (subject_id, mrh.clone_for_slab( from_slabref, to_slab, false )  ))
        }).collect();

        RelationSlotSubjectHead(new)
    }
}

impl Deref for RelationSlotSubjectHead {
    type Target = HashMap<RelationSlotId,(SubjectId,MemoRefHead)>;
    fn deref(&self) -> &HashMap<RelationSlotId,(SubjectId,MemoRefHead)> {
        &self.0
    }
}
