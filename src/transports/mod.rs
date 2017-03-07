//! Provides the framework and implementations for communications modules,
//! pluggable transports that allow connections between slabs. A `Transport` knows how to make 
//! `Transmitter`s which can be used to send `Memo`s.

use network::{Transmitter, MinkowskiTransmitter};
use std::sync::Arc;
use network::{Simulator, SlabRef};
use memo::Memo;

pub trait Transport {
    fn make_transmitter(&self) -> Box<Transmitter>;
}

pub struct SimulatorTransport {
    simulator: Arc<Simulator>
}

impl SimulatorTransport {
    pub fn new(s: Arc<Simulator>) -> Self {
        Self { simulator: s }
    }
}

impl Transport for SimulatorTransport {
    fn make_transmitter(&self) -> Box<Transmitter> {
        unimplemented!()
    }
}