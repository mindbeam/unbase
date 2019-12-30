use crate::network::{SlabRef, TransportAddress, Transport, TransmitterArgs, Transmitter};
use crate::slab::{SlabHandle, MemoRef};
use crate::network::transmitter::DynamicDispatchTransmitter;
use crate::Network;
use std::fmt;
use async_trait::async_trait;

use crate::util::simulator::{Simulator, SimEvent, Point3};
// TODO: determine how to account for execution time in a deterministic way
// suggest each operation be assigned a delay factor, such that some or all resultant events are deterministically delayed
pub struct MemoPayload {
    from_slabref:  SlabRef,
    dest:          SlabHandle,
    memoref:       MemoRef
}

#[async_trait]
impl SimEvent for MemoPayload {
    #[tracing::instrument]
    async fn deliver(self) {
        let slabref = self.dest.agent.localize_slabref(&self.from_slabref);
        self.dest.agent.localize_memoref( &self.memoref, &slabref, true );
    }
}

impl fmt::Debug for MemoPayload {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("MemoPayload")
            .field("dest", &self.dest.my_ref.slab_id)
            .field("memo", &self.memoref.id)
            .finish()
    }
}

impl Transport for Simulator <MemoPayload> {
    fn is_local (&self) -> bool {
        true
    }
    fn make_transmitter (
        &self,
        args: &TransmitterArgs,
    ) -> Option<Transmitter> {
        if let TransmitterArgs::Local(ref slab) = *args {
            let tx = SimulatorTransmitter{
                source_point: Point3 { x: 1000, y: 1000, z: 1000 }, // TODO: move this - not appropriate here
                dest_point: Point3 { x: 1000, y: 1000, z: 1000 },
                simulator: (*self).clone(),
                dest: (*slab).clone()
            };
            Some(Transmitter::new(args.get_slab_id(), Box::new(tx)))
        }else{
            None
        }

    }
    fn bind_network(&self, _net: &Network) {}
    fn unbind_network(&self, _net: &Network) {}
    fn get_return_address  ( &self, address: &TransportAddress ) -> Option<TransportAddress> {
        if let TransportAddress::Local = *address {
            Some(TransportAddress::Local)
        }else{
            None
        }
    }
}

pub struct SimulatorTransmitter{
    pub source_point: Point3,
    pub dest_point: Point3,
    pub simulator: Simulator<MemoPayload>,
    pub dest: SlabHandle
}

impl DynamicDispatchTransmitter for SimulatorTransmitter {
    #[tracing::instrument]
    fn send (&self, from_slabref: &SlabRef, memoref: MemoRef){

        let evt = MemoPayload {
            from_slabref: from_slabref.clone(),
            dest: self.dest.clone(),
            memoref: memoref
        };

        self.simulator.add_event( evt, &self.source_point, &self.dest_point);
    }
}

impl fmt::Debug for SimulatorTransmitter{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SimulatorTransmitter")
            .field("dest", &self.dest.my_ref.slab_id)
            .finish()
    }
}