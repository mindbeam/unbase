use std::fmt;
use super::*;
use std::sync::{Arc,Mutex};
use crate::slab::*;
use itertools::partition;
use crate::network::*;
use tracing::debug;

// Minkowski stuff: Still ridiculous, but necessary for our purposes.
pub struct XYZPoint{
    pub x: i64,
    pub y: i64,
    pub z: i64
}
pub struct MinkowskiPoint {
    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub t: u64
}

pub trait SimEventPayload {
    fn fire(self);
}

pub struct MemoDelivery {
    from_slabref:  SlabRef,
    dest:          SlabHandle,
    memoref:       MemoRef
}

struct SimEvent<SimEventPayload> {
    _source_point: MinkowskiPoint,
    dest_point:    MinkowskiPoint,
    payload: SimEventPayload
}

impl <P: SimEventPayload> SimEvent<P> {
    pub fn fire (self) {
        self.payload.fire();
    }
}
impl SimEventPayload for MemoDelivery {
    fn fire (self) {

        /* let memo = &self.memoref.get_memo_if_resident().unwrap();
        println!("Simulator.deliver FROM {} TO {} -> {}({:?}): {:?} {:?} {:?}",
            &self.from_slabref.slab_id,
            &to_slab.id,
            &self.memoref.id,
            &self.memoref.subject_id,
            &memo.body,
            &memo.parents.memo_ids(),
            &self.memoref.peerlist.read().unwrap().slab_ids()
        );*/
        let slabref = self.dest.agent.localize_slabref(&self.from_slabref);
        println!("# SimEvent.deliver MEMO {} from SLAB {} to SLAB {}", self.memoref.id, slabref.slab_id, self.dest.my_ref.slab_id );

        self.dest.agent.localize_memoref( &self.memoref, &slabref, true );

        // we all have to learn to deal with loss sometime
    }
}
impl <P: std::fmt::Debug> fmt::Debug for SimEvent<P>{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SimEvent")
            .field("t", &self.dest_point.t )
            .field("payload", &self.payload)
            .finish()
    }
}

impl fmt::Debug for MemoDelivery {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("MemoDelivery")
            .field("dest", &self.dest.my_ref.slab_id)
            .field("memo", &self.memoref.id)
            .finish()
    }
}

pub struct Simulator<P: SimEventPayload> {
    shared: Arc<Mutex<SimulatorInternal<P>>>,
    speed_of_light: u64,
}

impl <P:SimEventPayload> Clone for Simulator<P>{
    fn clone(&self) -> Self {
        Self{
            shared: self.shared.clone(),
            speed_of_light: self.speed_of_light
        }
    }
}
struct SimulatorInternal<P: SimEventPayload> {
    clock: u64,
    queue: Vec<SimEvent<P>>
}

impl <P: SimEventPayload> Simulator<P> {
    pub fn new() -> Self {
        Simulator {
            speed_of_light: 1, // 1 distance unit per time unit
            shared: Arc::new(Mutex::new(
                SimulatorInternal {
                    clock: 0,
                    queue: Vec::new()
                }
            ))
        }
    }

    fn add_event(&self, event: SimEvent<P>) {
        let mut shared = self.shared.lock().unwrap();
        shared.queue.push(event);
//        let seek = event.dest_point.t;
//        let idx = s.binary_search(&num).unwrap_or_else(|x| x);
//        s.insert(idx, num);
//        shared.queue.binary_search_by(|probe| probe.dest_point.t.cmp(&seek));
//        shared.queue.push(event);
    }
    pub fn get_clock(&self) -> u64 {
        self.shared.lock().unwrap().clock
    }

    #[tracing::instrument(level = "debug")]
    pub fn advance_clock (&self, ticks: u64) {
        debug!("advancing clock {} ticks", ticks);
        let t;
        let events : Vec<SimEvent<P>>;
        {
            let mut shared = self.shared.lock().unwrap();
            shared.clock += ticks;
            t = shared.clock;

            let split_index = partition(&mut shared.queue, |evt| evt.dest_point.t <= t );

            debug!(%split_index);
            events = shared.queue.drain(0..split_index).collect();
        }
        for event in events {
            event.fire();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn delivery_order() {
        let sim = Simulator::new();

    }
}

impl <P: SimEventPayload + fmt::Debug> fmt::Debug for Simulator<P> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let shared = self.shared.lock().unwrap();
        fmt.debug_struct("Simulator")
            .field("clock",&shared.clock)
            .field("queue", &shared.queue)
            .finish()
    }
}

impl Transport for Simulator<MemoDelivery> {
    fn is_local (&self) -> bool {
        true
    }
    fn make_transmitter (&self, args: &TransmitterArgs ) -> Option<Transmitter> {
        if let TransmitterArgs::Local(ref slab) = *args {
            let tx = SimulatorTransmitter{
                source_point: XYZPoint{ x: 1000, y: 1000, z: 1000 }, // TODO: move this - not appropriate here
                dest_point: XYZPoint{ x: 1000, y: 1000, z: 1000 },
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
    pub source_point: XYZPoint,
    pub dest_point: XYZPoint,
    pub simulator: Simulator<MemoDelivery>,
    pub dest: SlabHandle
}

impl DynamicDispatchTransmitter for SimulatorTransmitter {
    fn send (&self, from_slabref: &SlabRef, memoref: MemoRef){
        let ref q = self.source_point;
        let ref p = self.dest_point;

        let source_point = MinkowskiPoint {
            x: q.x,
            y: q.y,
            z: q.z,
            t: self.simulator.get_clock()
        };

        let distance = (( (q.x - p.x)^2 + (q.y - p.y)^2 + (q.z - p.z)^2 ) as f64).sqrt();

        let dest_point = MinkowskiPoint {
            x: p.x,
            y: p.y,
            z: p.z,
            t: source_point.t + ( distance as u64 * self.simulator.speed_of_light ) + 1 // add 1 to ensure nothing is instant
        };

        let evt = SimEvent {
            _source_point: source_point,
            dest_point: dest_point,
            payload: MemoDelivery {
                from_slabref: from_slabref.clone(),
                dest: self.dest.clone(),
                memoref: memoref
            }
        };

        self.simulator.add_event( evt );
    }
}
