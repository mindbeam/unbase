/*
    SlabRef intends to provide an abstraction for refering to a remote slab.
    Posessing a SlabRef does not confer ownership, or even imply locality.
    It does however provide us with a way to refer to a slab abstractly,
    and a means of getting messages to it.

    I labored a fair bit about whether this is materially different from
    the sender itself, but I think it is important, at least conceptually.
    Also, the internals of the sender could vary dramatically, whereas the
    SlabRef can continue to serve its purpose without material change.
*/

pub mod serde;

use std::mem;
use std::fmt;
use slab::{Slab,SlabId};
use super::memo::Memo;
use std::sync::{Arc,Mutex};
use std::sync::atomic;
use network::{TransportAddress,Transmitter};

/// A reference to a Slab
///
/// The referenced slab may be resident locally or Remotely
#[derive(Clone)]
pub struct SlabRef(pub Arc<SlabRefInner>);
pub struct SlabRefInner {
    pub to_slab_id: SlabId,
    pub owning_slab_id: SlabId, // for assertions only?
    pub presence: Mutex<Vec<SlabPresence>>,
    pub tx: Mutex<Transmitter>,
    pub return_address: atomic::AtomicPtr<TransportAddress>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SlabAnticipatedLifetime{
    Ephmeral,
    Session,
    Long,
    VeryLong,
    Unknown
}

/// SlabPresence represents the expected reachability of a given Slab
/// Including Transport address and anticipated lifetime
#[derive(Clone, Deserialize)]
pub struct SlabPresence{
    pub slab_id: SlabId,
    pub address: TransportAddress,
    pub lifetime: SlabAnticipatedLifetime
}

impl SlabRef{
    //pub fn new (to_slab_id: SlabId, owning_slab_id: SlabId, presence: Vec<Slab) -> SlabRef {
    //}
    pub fn send_memo (&self, from: &SlabRef, memo: Memo) {
        println!("# SlabRef({},{}).send_memo({})", self.0.owning_slab_id, self.0.to_slab_id, memo.id );
        self.0.tx.lock().unwrap().send(from, memo);

    }

    pub fn get_return_address(&self) -> TransportAddress {
        *(self.0.return_address.load(atomic::Ordering::Relaxed)).clone()
    }
    pub fn apply_presence ( &mut self, presence: &SlabPresence ) -> bool {
        let list = self.0.presence.lock().unwrap();
        for p in list.iter_mut(){
            if p == presence {
                mem::replace(p,presence.clone()); // Update anticipated liftime
                return false;
            }
        }
        list.push(presence.clone());
        return true
    }
    pub fn compare(&self, other: &SlabRef) -> bool {
        // When comparing equality, we can skip the transmitter
        self.0.to_slab_id == other.0.to_slab_id && *self.0.presence.lock().unwrap() == *other.0.presence.lock().unwrap()
    }
}

impl PartialEq for SlabPresence {
    fn eq(&self, other: &SlabPresence) -> bool {
        // When comparing equality, we can skip the anticipated lifetime
        self.slab_id == other.slab_id && self.address == other.address
    }
}
impl fmt::Debug for SlabRef {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SlabRef")
            .field("owning_slab_id", &self.0.to_slab_id)
            .field("to_slab_id",     &self.0.to_slab_id)
            .field("presence",       &self.0.presence.lock().unwrap())
            .finish()
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
impl Drop for SlabRefInner{
    fn drop(&mut self) {
        println!("# SlabRefInner({},{}).drop",self.owning_slab_id, self.to_slab_id);
    }
}
