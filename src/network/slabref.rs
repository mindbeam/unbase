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
use super::{Network,Transmitter};
use slab::{Slab,WeakSlab,SlabId};
use memo::Memo;
use std::sync::Arc;
use serde::ser::*;

#[derive(Clone)]
pub struct SlabRef {
    pub slab_id: SlabId,
    inner: Arc<SlabRefInner>
}
struct SlabRefInner {
    slab_id: SlabId,
    tx: Transmitter,
    _slab: WeakSlab
}

impl SlabRef{
    pub fn new_from_memo ( _memo: &Memo, _net: &Network ) -> SlabRef {
        // We just received a memo talking about the presence of a remote slab
        // I assume we'll hear about this from a memo somehow
        // Owing largely due to the fact that everything is a Memo :p

        unimplemented!();
    }
    pub fn new_from_slab ( slab: &Slab, net: &Network ) -> SlabRef {

        // TODO: Think about how a serialized slabref will create it's transmitters
        let tx = net.get_local_transmitter( &slab );

        SlabRef {
            slab_id: slab.id,
            inner: Arc::new (SlabRefInner {
                slab_id: slab.id,
                tx: tx,
                _slab: slab.weak() // for future use when we're actually communicating to resident slabs directly
            })
        }
    }

    pub fn send_memo (&self, from: &SlabRef, memo: Memo) {
        println!("# SlabRef({}).send_memo({})", self.slab_id, memo.id );
        self.inner.tx.send(from, memo);
    }
}

impl fmt::Debug for SlabRef {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SlabRef")
            .field("slab_id", &self.inner.slab_id)
            .finish()
    }
}
impl Serialize for SlabRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.slab_id.to_string());
        seq.serialize_element(&"127.0.0.1:12345".to_string());
        seq.end()

    }
}
