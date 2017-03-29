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

use std::fmt;
use super::*;
use slab::{Slab,SlabId};
use memo::Memo;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SlabAnticipatedLifetime{
    Ephmeral,
    Session,
    Long,
    VeryLong,
    Unknown
}

#[derive(Clone, Deserialize, PartialEq)]
pub struct SlabPresence{
    pub slab_id: SlabId,
    pub address: TransportAddress,
    pub lifetime: SlabAnticipatedLifetime
}

#[derive(Clone)]
pub struct SlabRef {
    pub slab_id: SlabId,
    pub presence: SlabPresence,
    inner: Arc<SlabRefInner>
}
struct SlabRefInner {
    slab_id: SlabId,
    local_return_address: Option<TransportAddress>,
    tx: Transmitter
}

impl SlabRef{
    pub fn new_from_presence ( presence: &SlabPresence, net: &Network ) -> SlabRef {

        match presence.address {
            TransportAddress::Simulator  => {
                panic!("Invalid - Cannot create simulator slabref from presence")
            }
            TransportAddress::Local      => {
                panic!("Invalid - Cannot create local slabref from presence")
            }
            _ => { }
        };

        let args = TransmitterArgs::Remote( &presence.slab_id, &presence.address );
        let tx = net.get_transmitter( args ).expect("new_from_presence net.get_transmitter");
        let maybe_local_return_address = net.get_return_address( &presence.address );

        SlabRef {
            slab_id: presence.slab_id,
            presence: presence.clone(),
            inner: Arc::new (SlabRefInner {
                slab_id: presence.slab_id,
                local_return_address: maybe_local_return_address,
                tx: tx
            })
        }
    }
    pub fn new_from_slab ( slab: &Slab, net: &Network ) -> SlabRef {

        let tx = net.get_transmitter( TransmitterArgs::Local(&slab) ).expect("new_from_slab net.get_transmitter");

        SlabRef {
            slab_id: slab.id,
            presence: SlabPresence{
                slab_id: slab.id,
                address: TransportAddress::Local,
                lifetime: SlabAnticipatedLifetime::Unknown
            },
            inner: Arc::new (SlabRefInner {
                slab_id: slab.id,
                local_return_address: Some(TransportAddress::Local),
                tx: tx
            })
        }
    }

    pub fn send_memo (&self, from: &SlabRef, memo: Memo) {
        println!("# SlabRef({}).send_memo({})", self.slab_id, memo.id );
        self.inner.tx.send(from, memo);
    }

    pub fn get_local_return_address(&self) -> &Option<TransportAddress> {
        &self.inner.local_return_address
    }
}

impl PartialEq for SlabRef {
    fn eq(&self, other: &SlabRef) -> bool {
        // When comparing equality, we can skip the transmitter
        self.slab_id == other.slab_id && self.presence == other.presence
    }
}

impl fmt::Debug for SlabRef {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SlabRef")
            .field("slab_id", &self.inner.slab_id)
            .field("address", &self.presence.address.to_string())
            .field("lifetime", &self.presence.lifetime)
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
