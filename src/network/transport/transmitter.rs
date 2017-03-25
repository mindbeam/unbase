
use std::sync::mpsc;
use super::*;
use super::simulator::SimulatorTransmitter;
use std::thread;

/// A trait for transmitters to implement
pub trait DynamicDispatchTransmitter {
    /// Transmit a memo to this Transmitter's recipient
    fn send (&self, from: &SlabRef, memo: Memo);
}

enum TransmitterInternal {
    Local(Mutex<mpsc::Sender<(SlabId,Memo)>>),
    Simulator(SimulatorTransmitter),
    Dynamic(Box<DynamicDispatchTransmitter + Send + Sync>)
}

pub struct Transmitter {
    internal: TransmitterInternal
}

impl Transmitter {
    /// Create a new transmitter associated with a local slab.
    pub fn new_local( tx: Mutex<mpsc::Sender<(SlabId,Memo)>> ) -> Self {
        Self {
            internal: TransmitterInternal::Local( tx )
        }
    }
    /// Create a new transmitter associated with a local simulator transmitter.
    pub fn new_simulated(sim_tx: SimulatorTransmitter) -> Self {
        Self {
            internal: TransmitterInternal::Simulator(sim_tx)
        }
    }
    /// Create a new transmitter capable of using any dynamic-dispatch transmitter.
    pub fn new(dyn: Box<DynamicDispatchTransmitter + Send + Sync>) -> Self {
        Self {
            internal: TransmitterInternal::Dynamic(dyn)
        }
    }
    /// Send a Memo over to the target of this transmitter
    pub fn send(&self, from: &SlabRef, memo: Memo) {
        use self::TransmitterInternal::*;
        match self.internal {
            Local(ref tx) => {
                tx.lock().unwrap().send((from.slab_id,memo)).expect("local transmitter send")
            }
            Simulator(ref tx) => {
                tx.send(from, memo);
            }
            Dynamic(ref tx) => {
                tx.send(from,memo)
            }
        }
    }
}
