use std::sync::{Arc,Mutex};
use itertools::partition;
use std::fmt;
use tracing::debug;

// Minkowski stuff: Still ridiculous, but necessary for our purposes.
pub struct Point3 {
    pub x: i64,
    pub y: i64,
    pub z: i64
}
#[derive(PartialEq, Debug)]
pub struct Point4 {
    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub t: u64
}

pub trait SimEventPayload {
    fn fire(self);
}

pub struct SimEvent<SimEventPayload> {
    pub source_point: Point4,
    pub dest_point: Point4,
    pub payload: SimEventPayload
}

impl <P: SimEventPayload> SimEvent<P> {
    pub fn fire (self) {
        self.payload.fire();
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

pub struct Simulator<P: SimEventPayload> {
    shared: Arc<Mutex<SimulatorInternal<P>>>,
    pub speed_of_light: u64,
}

impl <P: SimEventPayload> Clone for Simulator<P>{
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

impl <P: SimEventPayload + fmt::Debug> Simulator<P> {
    pub fn new() -> Self {
        Simulator {
            speed_of_light: 1, // 1 distance unit per time unit
            shared: Arc::new(Mutex::new(
                SimulatorInternal::<P> {
                    clock: 0,
                    queue: Vec::new()
                }
            ))
        }
    }

    pub fn add_event(&self, event: SimEvent<P>) {
        let mut shared = self.shared.lock().unwrap();

        let seek = event.dest_point.t;
        let idx = shared.queue.binary_search_by(|probe| probe.dest_point.t.cmp(&seek)).unwrap_or_else(|x| x);
        shared.queue.insert(idx, event);
    }
    pub fn get_clock(&self) -> u64 {
        self.shared.lock().unwrap().clock
    }

    #[tracing::instrument(level = "debug")]
    pub fn advance_clock (&self, ticks: u64) {
        debug!("advancing clock {} ticks", ticks);
        //TODO: advance only one tick at a time (skipping forward when gaps exist)
        //TODO: determine how to simulate processing time for the event.fire, and how/if to consider that slab busy until processing is completed
        //TODO: add automatic clock advancement mode
        let events : Vec<SimEvent<P>> = self.advance_and_fetch(ticks);
        for event in events {
            event.fire();
        }
    }
    fn advance_and_fetch (&self, ticks: u64) -> Vec<SimEvent<P>> {
        let mut shared = self.shared.lock().unwrap();
        shared.clock += ticks;
        let t = shared.clock;

        let split_index = partition(&mut shared.queue, |evt| evt.dest_point.t <= t );

        debug!(%split_index);
        shared.queue.drain(0..split_index).collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug)]
    struct DummyPayload {}
    impl SimEventPayload for DummyPayload {
        fn fire(self) {}
    }

    #[test]
    fn delivery_order() {
        let sim = Simulator::<DummyPayload>::new();
        sim.add_event(SimEvent {
            source_point: Point4 { x: 0, y: 0, z: 0, t: 0 },
            dest_point:    Point4 { x: 1, y: 0, z: 0, t: 1 },
            payload:       DummyPayload{},
        });
        sim.add_event(SimEvent {
            source_point: Point4 { x: 0, y: 0, z: 0, t: 0 },
            dest_point:    Point4 { x: -1, y: 0, z: 0, t: 1 },
            payload:       DummyPayload{},
        });
        sim.add_event(SimEvent {
            source_point: Point4 { x: 0, y: 0, z: 0, t: 0 },
            dest_point:    Point4 { x: 3, y: 0, z: 0, t: 3 },
            payload:       DummyPayload{},
        });
        sim.add_event(SimEvent {
            source_point: Point4 { x: 0, y: 0, z: 0, t: 0 },
            dest_point:    Point4 { x: 0, y: 1, z: 0, t: 1 },
            payload:       DummyPayload{},
        });
        sim.add_event(SimEvent {
            source_point: Point4 { x: 0, y: 0, z: 0, t: 1 },
            dest_point:    Point4 { x: 2, y: 0, z: 0, t: 2 },
            payload:       DummyPayload{},
        });

        let seq : Vec<u64> = sim.shared.lock().unwrap().queue.iter().map(|e| e.dest_point.t ).collect();
        assert_eq!(seq, vec![1u64,1,1,2,3]);

        let dests : Vec<Point4>= sim.advance_and_fetch(1).into_iter().map(|e| e.dest_point ).collect();
        // NOTE: at present, we are reversing the event order for identical timeslots.
        // In theory this shouldn't matter, as no communications should arrive in less than one clock tick
        assert_eq!( dests , vec![Point4 { x: -1, y: 0, z: 0, t: 1 }, Point4 { x: 0, y: 1, z: 0, t: 1 }, Point4 { x: 1, y: 0, z: 0, t: 1 }]);

        let dests : Vec<Point4>= sim.advance_and_fetch(1).into_iter().map(|e| e.dest_point ).collect();
        assert_eq!( dests , vec![Point4 { x: 2, y: 0, z: 0, t: 2 }] );

        let dests : Vec<Point4>= sim.advance_and_fetch(1).into_iter().map(|e| e.dest_point ).collect();
        assert_eq!( dests , vec![Point4 { x: 3, y: 0, z: 0, t: 3 }] );
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