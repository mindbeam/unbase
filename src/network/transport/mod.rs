//! Provides the framework and implementations for communications modules,
//! pluggable transports that allow connections between slabs. A `Transport` knows how to make
//! `Transmitter`s which can be used to send `Memo`s.

mod transmitter;
mod simulator;
mod udp;

pub use self::udp::Transport_UDP;
pub use self::simulator::Simulator;
pub use self::transmitter::{Transmitter, DynamicDispatchTransmitter};

use network::*;
use slab::Slab;
use memo::Memo;

pub enum TransmitterArgs<'a>{
    Local(&'a Slab),
    Remote(&'a String)
}

pub trait Transport {
    fn make_transmitter(  &self, args: TransmitterArgs  ) -> Result<Transmitter,String>;
    fn is_local        (  &self ) -> bool;
    fn bind_network    (  &self, &Network );
}
