pub mod serde;
use super::super::*;
use slab::SlabId;

#[derive(Clone, Serialize)]
pub struct Packet {
    pub to_slab_id: SlabId,
    pub from_slab_id: SlabId,
    pub memo: Memo
}
