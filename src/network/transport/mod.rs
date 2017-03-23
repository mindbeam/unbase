//! Provides the framework and implementations for communications modules,
//! pluggable transports that allow connections between slabs. A `Transport` knows how to make
//! `Transmitter`s which can be used to send `Memo`s.

mod transmitter;
//mod local_direct;
mod simulator;
mod udp;

pub use self::udp::*;
pub use self::simulator::Simulator;
pub use self::transmitter::{Transmitter, DynamicDispatchTransmitter};

use network::*;
use slab::Slab;
use memo::Memo;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TransportAddress{
    Simulator,
    Local,
    UDP(TransportAddressUDP),
    UDT,
    WebRTP,
    SCMP,
    Bluetooth,
    ShamefulTCP // SHAME! SHAME! SHAME! ( yes, I _really_ want to discourage people from using TCP )
}

#[derive(Debug)]
pub enum TransmitterArgs<'a>{
    Local(&'a Slab),
    Remote(&'a SlabId, &'a TransportAddress)
}

pub trait Transport {
    fn make_transmitter(  &self, args: &TransmitterArgs  ) -> Option<Transmitter>;
    fn is_local        (  &self ) -> bool;
    fn bind_network    (  &self, &Network );
    fn get_return_address  ( &self, &TransportAddress ) -> Option<TransportAddress>;
}

impl TransportAddress {

}
