
use std::sync::mpsc;
use super::*;
use slab::*;

/// A trait for transmitters to implement
pub trait DynamicDispatchTransmitter {
    /// Transmit a memo to this Transmitter's recipient
    fn send (&self, from: &SlabRef, memoref: MemoRef);
}

enum TransmitterInternal {
    Local(Mutex<mpsc::Sender<(SlabRef,MemoRef)>>),
    Dynamic(Box<DynamicDispatchTransmitter + Send + Sync>),
    Blackhole
}

pub struct Transmitter {
    internal: TransmitterInternal
}

impl Transmitter {
    /// Create a new transmitter associated with a local slab.
    pub fn new_local( tx: Mutex<mpsc::Sender<(SlabRef,MemoRef)>> ) -> Self {
        Self {
            internal: TransmitterInternal::Local( tx )
        }
    }
    pub fn new_blackhole() -> Self {
        Self {
            internal: TransmitterInternal::Blackhole
        }
    }
    /// Create a new transmitter capable of using any dynamic-dispatch transmitter.
    pub fn new(dyn: Box<DynamicDispatchTransmitter + Send + Sync>) -> Self {
        Self {
            internal: TransmitterInternal::Dynamic(dyn)
        }
    }
    /// Send a Memo over to the target of this transmitter
    pub fn send(&self, from: &SlabRef, memoref: MemoRef) {
        println!("Transmitter.send({:?}, {:?})", from, memoref );

        use self::TransmitterInternal::*;
        match self.internal {
            Local(ref tx) => {
                //println!("CHANNEL SEND from {}, {:?}", from.slab_id, memo);
                // TODO - stop assuming that this is resident on the sending slab just because we're sending it
                // TODO - lose the stupid lock on the transmitter
                tx.lock().unwrap().send((from.clone(),memoref)).expect("local transmitter send")
            }
            Dynamic(ref tx) => {
                tx.send(from,memoref)
            }
            Blackhole => {
                println!("WARNING! Transmitter Blackhole transmitter used. from {:?}, memoref {:?}", from, memoref );
            }
        }
    }
}

impl Drop for TransmitterInternal{
    fn drop(&mut self) {
        println!("# TransmitterInternal().drop");
    }
}
