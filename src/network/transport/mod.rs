//! Provides the framework and implementations for communications modules,
//! pluggable transports that allow connections between slabs. A `Transport` knows how to make
//! `Transmitter`s which can be used to send `Memo`s.

mod simulator;

pub use self::simulator::Simulator;

use network::SlabRef;
use slab::Slab;
use memo::Memo;
use std::sync::Arc;

pub enum TransmitterArgs<'a>{
    Local(&'a Slab),
    Remote(&'a String)
}
pub trait Transport {
    fn make_transmitter(  &self, args: TransmitterArgs  ) -> Result<Arc<Transmitter>,String>;
    fn is_local        (  &self ) -> bool;
}

pub trait Transmitter {
    /// Transmit a memo to this Transmitter's recipient
    fn send (&self, from: &SlabRef, memo: Memo);
}
