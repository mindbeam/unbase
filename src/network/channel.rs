use memo::Memo;

//use slab::WeakSlab;
use super::*;
//use super::{Simulator,XYZPoint};

// INITIAL channel implementation - A deterministic channel sufficient for testing only
// This is not intended to be high performance
// It does intend to execute in real time
// It indends only to provide a simplistic minkowski-style spatial model of communication
// which is decidedly bogus - but useful for initial testing purposes

// TODO: source_point seems dubious. Need to clarify whether this channel is specific to a origin & recipient
//       or just the recipient
pub struct Sender{
    pub source_point: XYZPoint,
    pub dest_point: XYZPoint,
    pub simulator: Simulator,
    pub dest: WeakSlab
}

impl Sender {
    pub fn send (&self, from: &SlabRef, memo: Memo){
        self.simulator.send_memo( &self, from, memo );
    }
}
