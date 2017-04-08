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
use super::common_structs::*;
use super::memo::Memo;
use super::memoref::MemoRef;
use std::sync::{Arc,Mutex};
use std::sync::atomic;
use network::{TransportAddress,Transmitter};

/// A reference to a Slab
///
/// The referenced slab may be resident locally or Remotely
#[derive(Clone)]
pub struct SlabRef(pub Arc<SlabRefInner>);
pub struct SlabRefInner {
    pub slab_id: SlabId,
    pub owning_slab_id: SlabId, // for assertions only?
    pub presence: Mutex<Vec<SlabPresence>>,
    pub tx: Mutex<Transmitter>,
    pub return_address: atomic::AtomicPtr<TransportAddress>,
}

impl SlabRef{
    //pub fn new (to_slab_id: SlabId, owning_slab_id: SlabId, presence: Vec<Slab) -> SlabRef {
    //}
    pub fn send (&self, from: &SlabRef, memoref: &MemoRef ) {
        println!("# SlabRef({},{}).send_memo({})", self.0.owning_slab_id, self.0.slab_id, memoref.id );

        if let Some(memo) = memoref.get_memo_if_resident() {
            self.0.tx.lock().unwrap().send(from, memo);
        }else{
            // NOTE: we should actually implement this
            //       it is a totally reasonable use case that we might want to send a memo
            //       to a remote slab that we do not ourselves have
            unimplemented!();
        }
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
    pub fn get_presence(&self) -> Vec<SlabPresence> {
        self.0.presence.lock().unwrap().clone()
    }
    pub fn compare(&self, other: &SlabRef) -> bool {
        // When comparing equality, we can skip the transmitter
        self.0.slab_id == other.0.slab_id && *self.0.presence.lock().unwrap() == *other.0.presence.lock().unwrap()
    }
}

impl fmt::Debug for SlabRef {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SlabRef")
            .field("owning_slab_id", &self.0.slab_id)
            .field("to_slab_id",     &self.0.slab_id)
            .field("presence",       &self.0.presence.lock().unwrap())
            .finish()
    }
}

impl Drop for SlabRefInner{
    fn drop(&mut self) {
        println!("# SlabRefInner({},{}).drop",self.owning_slab_id, self.slab_id);
    }
}
