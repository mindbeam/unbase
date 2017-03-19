//! Provides the framework and implementations for communications modules,
//! pluggable transports that allow connections between slabs. A `Transport` knows how to make
//! `Transmitter`s which can be used to send `Memo`s.

mod transmitter;
mod simulator;
mod udp;

pub use self::udp::*;
pub use self::simulator::Simulator;
pub use self::transmitter::{Transmitter, DynamicDispatchTransmitter};

use network::*;
use slab::Slab;
use memo::Memo;
use serde::ser::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum SlabAnticipatedLifetime{
    Ephmeral,
    Session,
    Long,
    VeryLong,
    Unknown
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TransportAddress{
    Local,
    UDP(TransportAddressUDP),
    UDT,
    WebRTP,
    SCMP,
    Bluetooth,
    ShamefulTCP // SHAME! SHAME! SHAME! ( yes, I _really_ want to discourage people from using TCP )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlabPresence{
    pub slab_id: SlabId,
    pub transport_address: TransportAddress,
    pub anticipated_lifetime: SlabAnticipatedLifetime
}

pub enum TransmitterArgs<'a>{
    Local(&'a Slab),
    Remote(&'a SlabId, TransportAddress)
}

pub trait Transport {
    fn make_transmitter(  &self, args: TransmitterArgs  ) -> Result<Transmitter,String>;
    fn is_local        (  &self ) -> bool;
    fn bind_network    (  &self, &Network );
}
