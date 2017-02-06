use memo::Memo;

use itertools::partition;
use std::sync::{Arc,Mutex};
use slab::{Slab,WeakSlab,SlabId};

// INITIAL channel implementation - A deterministic channel sufficient for testing only
// This is not intended to be high performance
// It does intend to execute in real time
// It indends only to provide a simplistic minkowski-style spatial model of communication
// which is decidedly bogus - but useful for initial testing purposes

pub struct Sender{
    source_point: XYZPoint,
    dest_point: XYZPoint,
    oculus_dei: OculusDei,
    dest: WeakSlab
}

// A ridiculous name for a ridiculous thing
#[derive(Clone)]
pub struct OculusDei {
    shared: Arc<Mutex<InteriusOculusDei>>,
    speed_of_light: u64
}
struct InteriusOculusDei {
    clock: u64,
    queue: Vec<Event>
}

// Minkowski stuff: Still ridiculous, but necessary for our purposes.
pub struct XYZPoint{
    x: i64,
    y: i64,
    z: i64
}
pub struct MinkowskiPoint {
    x: i64,
    y: i64,
    z: i64,
    t: u64
}

struct Event {
    source_point: MinkowskiPoint,
    dest_point:   MinkowskiPoint,
    dest:         WeakSlab,
    memo:         Memo
}

impl Event {
    pub fn deliver (&self) {
        if let Some(slab) = self.dest.upgrade() {
            slab.put_memos(vec![self.memo])
        }
        // we all have to learn to deal with loss sometime
    }
}

impl Sender {
    pub fn send (&self, memo: Memo){

        let q = self.source_point;
        let p = self.dest_point;

        let source_point = MinkowskiPoint {
            x: q.x,
            y: q.y,
            z: q.z,
            t: self.oculus_dei.get_clock()
        };

        let distance = (( (q.x - p.x)^2 + (q.z - p.z)^2 + (q.z - p.z)^2 ) as f64).sqrt();


        let dest_point = MinkowskiPoint {
            x: p.x,
            y: p.y,
            z: p.z,
            t: source_point.t + ( distance as u64 * self.oculus_dei.speed_of_light ) + 1 // add 1 to ensure nothing is instant
        };

        let evt = Event {
            source_point: source_point,
            dest_point: dest_point,
            dest: self.dest.clone(),
            memo: memo
        };

        self.oculus_dei.add_event( evt );
    }
}

impl OculusDei {
    pub fn new () -> Self{
        OculusDei {
            speed_of_light: 1, // 1 distance unit per time unit
            shared: Arc::new(Mutex::new(
                InteriusOculusDei {
                    clock: 0,
                    queue: Vec::new()
                }
            ))
        }
    }
    pub fn add_event(&self, event: Event) {
        let mut shared = self.shared.lock().unwrap();
        shared.queue.push(event);
    }
    pub fn get_clock(&self) -> u64 {
        self.shared.lock().unwrap().clock
    }
    pub fn advance_clock (&self, ticks: u64) {
        let t;
        let events : Vec<Event>;
        {
            let mut shared = self.shared.lock().unwrap();
            shared.clock += ticks;
            t = shared.clock;

            let split_index = partition(&mut shared.queue, |evt| evt.dest_point.t >= t );
            events = shared.queue.drain(1..split_index).collect();
        }

        for event in events {
            event.deliver();
        }

    }
}
