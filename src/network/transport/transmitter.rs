
use std::sync::mpsc;
use super::*;

/// A trait for transmitters to implement
pub trait DynamicDispatchTransmitter {
    /// Transmit a memo to this Transmitter's recipient
    fn send (&self, from: &SlabRef, memo: Memo);
}

enum TransmitterInternal {
    Local(Mutex<mpsc::Sender<(SlabId,Memo)>>),
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
                println!("CHANNEL SEND from {}, {:?}", from.slab_id, memo);
                tx.lock().unwrap().send((from.slab_id,memo)).expect("local transmitter send")
            }
            Dynamic(ref tx) => {
                tx.send(from,memo)
            }
        }
    }
}
