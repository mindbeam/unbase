use memo::Memo;

//use slab::WeakSlab;
use super::*;
//use super::{Simulator,XYZPoint};
use transports::Transport;
use std::sync::Arc;

// TODO: source_point seems dubious. Need to clarify whether this channel is specific to a origin & recipient
//       or just the recipient

// Design
// Transmitters store at least a reciever and transport, and know how to send communications.

pub trait Transmitter {
    /// Transmit a memo to this Transmitter's recipient
    fn send (&self, from: &SlabRef, memo: Memo);
}

/// A deterministic channel sufficient for testing only. Not high performance, does not
/// work in real time. indends only to provide a simplistic minkowski-style spatial model of communication
/// which is decidedly bogus - but useful for initial testing purposes.
pub struct MinkowskiTransmitter{
    pub source_point: XYZPoint,
    pub dest_point: XYZPoint,
    pub transport: Arc<Transport>,
    pub dest: WeakSlab
}

impl MinkowskiTransmitter {
    pub fn new (source_point: XYZPoint, dest_point: XYZPoint, transport: Arc<Transport>, dest: WeakSlab) -> Self {
        Self {
            source_point: source_point,
            dest_point: dest_point,
            transport: transport,
            dest: dest
        }
    }
}

impl Transmitter for MinkowskiTransmitter {
    fn send (&self, from: &SlabRef, memo: Memo){
        unimplemented!()
    }
}
