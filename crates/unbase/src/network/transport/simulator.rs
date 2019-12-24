use crate::network::{SlabRef, TransportAddress, Transport, TransmitterArgs, Transmitter};
use crate::slab::{SlabHandle, MemoRef};
use crate::network::transmitter::DynamicDispatchTransmitter;
use crate::Network;
use std::fmt;

use unbase_test_util::simulator::{Simulator, SimEventPayload, Point3, Point4, SimEvent};
// TODO: determine how to account for execution time in a deterministic way
// suggest each operation be assigned a delay factor, such that some or all resultant events are deterministically delayed
pub struct MemoPayload {
    from_slabref:  SlabRef,
    dest:          SlabHandle,
    memoref:       MemoRef
}

impl SimEventPayload for MemoPayload {
    #[tracing::instrument]
    fn fire (self) {
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
    fn make_transmitter (&self, args: &TransmitterArgs ) -> Option<Transmitter> {
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
    fn send (&self, from_slabref: &SlabRef, memoref: MemoRef){
        let ref q = self.source_point;
        let ref p = self.dest_point;

        let source_point = Point4 {
            x: q.x,
            y: q.y,
            z: q.z,
            t: self.simulator.get_clock()
        };

        let distance = (( (q.x - p.x)^2 + (q.y - p.y)^2 + (q.z - p.z)^2 ) as f64).sqrt();

        let dest_point = Point4 {
            x: p.x,
            y: p.y,
            z: p.z,
            t: source_point.t + ( distance as u64 * self.simulator.speed_of_light ) + 1 // add 1 to ensure nothing is instant
        };

        let evt = SimEvent {
            source_point: source_point,
            dest_point: dest_point,
            payload: MemoPayload {
                from_slabref: from_slabref.clone(),
                dest: self.dest.clone(),
                memoref: memoref
            }
        };

        self.simulator.add_event( evt );
    }
}
