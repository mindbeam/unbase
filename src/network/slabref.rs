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


use std::fmt;
use network::Sender;
use slab::{Slab,WeakSlab,SlabId};
use memo::Memo;
use std::sync::Arc;

#[derive(Clone)]
pub struct SlabRef {
    inner: Arc<SlabRefInner>
}
struct SlabRefInner {
    slab_id: SlabId,
    sender: Sender,
    _slab: WeakSlab
}

impl SlabRef{
    pub fn new (slab: &Slab, sender: Sender ) -> SlabRef {
        SlabRef {
            inner: Arc::new (SlabRefInner {
                slab_id: slab.id,
                sender: sender,
                _slab: slab.weak() // for future use when we're actually communicating to resident slabs directly
            })
        }
    }

    pub fn send_memo (&self, from: &SlabRef, memo: Memo) {
        self.inner.sender.send(from, memo);
    }
}

impl fmt::Debug for SlabRef {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SlabRef")
            .field("slab_id", &self.inner.slab_id)
            .finish()
    }
}
