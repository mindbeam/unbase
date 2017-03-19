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
use super::*;
use slab::{Slab,WeakSlab,SlabId};
use memo::Memo;
use std::sync::Arc;
use serde::ser::*;

#[derive(Clone)]
pub struct SlabRef {
    pub slab_id: SlabId,
    pub address: TransportAddress,
    inner: Arc<SlabRefInner>
}
struct SlabRefInner {
    slab_id: SlabId,
    tx: Transmitter
}

impl SlabRef{
    pub fn new_from_presence ( presence: &SlabPresence, net: &Network ) -> SlabRef {

        let tx = net.get_remote_transmitter( presence.slab_id, presence.transport_address );

        SlabRef {
            slab_id: presence.slab_id,
            address: presence.transport_address,
            inner: Arc::new (SlabRefInner {
                slab_id: presence.slab_id,
                tx: tx
            })
        }
    }
    pub fn new_from_slab ( slab: &Slab, net: &Network ) -> SlabRef {

        // TODO: Think about how a serialized slabref will create it's transmitters
        let tx = net.get_local_transmitter( &slab );

        SlabRef {
            slab_id: slab.id,
            address: TransportAddress::Local,
            inner: Arc::new (SlabRefInner {
                slab_id: slab.id,
                tx: tx
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
        seq.serialize_element(&self.slab_id.to_string())?;
        seq.serialize_element(&"127.0.0.1:12345".to_string())?;
        seq.end()

    }
}
