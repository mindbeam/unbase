use memo::Memo;

use slab::WeakSlab;
use super::{Simulator,XYZPoint};

// INITIAL channel implementation - A deterministic channel sufficient for testing only
// This is not intended to be high performance
// It does intend to execute in real time
// It indends only to provide a simplistic minkowski-style spatial model of communication
// which is decidedly bogus - but useful for initial testing purposes

pub struct Sender{
    pub source_point: XYZPoint,
    pub dest_point: XYZPoint,
    pub simulator: Simulator,
    pub dest: WeakSlab
}

impl Sender {
    pub fn send (&self, memo: Memo){
        self.simulator.send_memo( &self, memo );
    }
}
