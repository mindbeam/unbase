
use std::fmt;
use super::*;
use std::sync::{Arc,Mutex};
use itertools::partition;

#[derive(Clone)]
pub struct Simulator {
    shared: Arc<Mutex<SimulatorInternal>>,
    speed_of_light: u64
}
struct SimulatorInternal {
    clock: u64,
    queue: Vec<SimEvent>
}

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
    from:          SlabRef,
    dest:          WeakSlab,
    memo:          Memo
}

impl SimEvent {
    pub fn deliver (self) {
        println!("# SimEvent.deliver {} to Slab {}", &self.memo.id, self.dest.id );
        if let Some(slab) = self.dest.upgrade() {
            slab.put_memos(MemoOrigin::Remote(&self.from), vec![self.memo])
        }
        // we all have to learn to deal with loss sometime
    }
}
impl fmt::Debug for SimEvent{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SimEvent")
            .field("dest", &self.dest.id )
            .field("memo", &self.memo.id )
            .field("t", &self.dest_point.t )
            .finish()
    }
}

impl Simulator {
    pub fn new () -> Self{
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
    pub fn send_memo (&self, sender: &Sender, from: &SlabRef, memo: Memo ){
        let ref q = sender.source_point;
        let ref p = sender.dest_point;

        let source_point = MinkowskiPoint {
            x: q.x,
            y: q.y,
            z: q.z,
            t: self.get_clock()
        };

        let distance = (( (q.x - p.x)^2 + (q.y - p.y)^2 + (q.z - p.z)^2 ) as f64).sqrt();

        let dest_point = MinkowskiPoint {
            x: p.x,
            y: p.y,
            z: p.z,
            t: source_point.t + ( distance as u64 * self.speed_of_light ) + 1 // add 1 to ensure nothing is instant
        };

        let evt = SimEvent {
            _source_point: source_point,
            dest_point: dest_point,
            from: from.clone(),
            dest: sender.dest.clone(),
            memo: memo
        };

        self.add_event( evt );
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
