
use std::fmt;
use super::*;
use std::sync::{Arc,Mutex};
use crate::slab::*;
use itertools::partition;
use crate::network::*;

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
struct SimEvent {
    _source_point: MinkowskiPoint,
    dest_point:    MinkowskiPoint,
    from_slabref:  SlabRef,
    dest:          SlabHandle,
    memoref:       MemoRef
}

impl SimEvent {
    pub fn deliver (self) {
        //println!("# SimEvent.deliver" );

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
        self.dest.agent.localize_memoref( &self.memoref, &slabref, true );

        // we all have to learn to deal with loss sometime
    }
}
impl fmt::Debug for SimEvent{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SimEvent")
            .field("dest", &self.dest.my_ref.slab_id )
            .field("memo", &self.memoref.id )
            .field("t", &self.dest_point.t )
            .finish()
    }
}

#[derive(Clone)]
pub struct Simulator {
    shared: Arc<Mutex<SimulatorInternal>>,
    speed_of_light: u64,
}
struct SimulatorInternal {
    clock: u64,
    queue: Vec<SimEvent>
}

impl Simulator {
    pub fn new() -> Self{
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

    fn add_event(&self, event: SimEvent) {
        let mut shared = self.shared.lock().unwrap();
        shared.queue.push(event);
    }
    pub fn get_clock(&self) -> u64 {
        self.shared.lock().unwrap().clock
    }
    pub fn advance_clock (&self, ticks: u64) {

        println!("# Simulator.advance_clock({})", ticks);

        let t;
        let events : Vec<SimEvent>;
        {
            let mut shared = self.shared.lock().unwrap();
            shared.clock += ticks;
            t = shared.clock;

            let split_index = partition(&mut shared.queue, |evt| evt.dest_point.t >= t );

            events = shared.queue.drain(0..split_index).collect();
        }
        for event in events {
            event.deliver();
        }
    }
}

impl fmt::Debug for Simulator {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let shared = self.shared.lock().unwrap();
        fmt.debug_struct("Simulator")
            .field("queue", &shared.queue)
            .finish()
    }
}

impl Transport for Simulator {
    fn is_local (&self) -> bool {
        true
    }
    fn make_transmitter (&self, args: &TransmitterArgs ) -> Option<Transmitter> {
        if let TransmitterArgs::Local(ref slab) = *args {
            let tx = SimulatorTransmitter{
                source_point: XYZPoint{ x: 1000, y: 1000, z: 1000 }, // TODO: move this - not appropriate here
                dest_point: XYZPoint{ x: 1000, y: 1000, z: 1000 },
                simulator: self.clone(),
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
    pub simulator: Simulator,
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
            from_slabref: from_slabref.clone(),

            dest: self.dest.clone(),
            memoref: memoref
        };

        self.simulator.add_event( evt );
    }
}
