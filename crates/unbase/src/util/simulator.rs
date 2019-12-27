use {
    futures::{
        future::RemoteHandle,
        stream::{
            self,
            Stream,
            StreamExt
        }
    },
    std::{
        fmt,
        future::{
            Future,
        },
        pin::Pin,
        sync::{Arc, Mutex},
        task::{Context, Poll, Waker},
    },
};
use tracing::{span,Level};

use itertools::partition;
use tracing::debug;

pub trait SimEventPayload {
    fn fire(self) -> Pin<Box<dyn Future<Output=()>>>;
}

pub struct Simulator<P: SimEventPayload> {
    shared: Arc<Mutex<SimulatorInternal<P>>>,
    runner: Arc<Mutex<Option<RemoteHandle<()>>>>,
}

impl <P: SimEventPayload> Clone for Simulator<P>{
    fn clone(&self) -> Self {
        Self{
            shared: self.shared.clone(),
            runner: self.runner.clone(),
        }
    }
}

impl <P: SimEventPayload + fmt::Debug> Simulator<P> {
    pub fn new() -> Self {
        Simulator {
            shared: Arc::new(Mutex::new(
                SimulatorInternal::<P> {
                    clock: 0,
                    queue: Vec::new(),
                    speed_of_light: 1, // 1 distance unit per time unit
                    woke: false,
                    waker: None,
                }
            )),
            runner: Arc::new(Mutex::new(None))
        }
    }
    /// If the simulator is currentlly applying events, no new events can be observed until the next tick
    // In the case that we want to model computational time as being more than 1 tick, we should queue the events with a different departure time
    pub fn add_event(&self, event: SimEvent<P>) {
        let mut shared = self.shared.lock().unwrap();

        let seek = event.dest_point.t;
        let idx = shared.queue.binary_search_by(|probe| probe.dest_point.t.cmp(&seek)).unwrap_or_else(|x| x);
        shared.queue.insert(idx, event);

        if let Some(waker) = shared.waker {
            if !shared.woke {
                waker.wake();
                shared.woke = true;
            }
        }
    }
    pub fn get_clock(&self) -> u64 {
        self.shared.lock().unwrap().clock
    }

    #[tracing::instrument(level = "debug")]
    pub async fn advance_clock (&self) {
        debug!("advancing clock");

        let events = {
            let shared = self.shared.lock().unwrap();
            shared.woke = false;
            shared.advance_and_fetch()
        };
        // Gotta fire the events outside of the lock

        if let Some(events) = events {
            stream::iter(events).for_each_concurrent(
                None,
                |rx| async move {
                    rx.payload.fire().await
                }
            ).await;
        }
    }
    fn tickstream (&self) -> TickStream<P> {
        TickStream {
            shared: self.shared.clone()
        }
    }
    pub fn start (&self) -> bool {
        let mut runner = self.runner.lock().unwrap();

        if runner.is_some() {
            return false;
        }

        let span = span!(Level::DEBUG, "Simulator Runner");

        let mut tickstream = self.tickstream();
        let handle: RemoteHandle<()> = crate::util::task::spawn_with_handle((async move || {
            let _guard = span.enter();

            // get a chunk of events
            while let Some(events) = tickstream.next().await {
                // run them all to completion without looking back in the queue
                stream::iter(events).for_each_concurrent(
                    None,
                    |rx| async move {
                        rx.payload.fire().await
                    }
                ).await;
            }

        })());

        *runner = Some(handle);

        true
    }
    pub fn stop (&self) -> bool {
        let mut runner = self.runner.lock().unwrap();
        if let Some(_handle) = runner.take() {
            return true;
        }
        return false;
    }
}

struct TickStream<P> {
    shared: Arc<Mutex<SimulatorInternal<P>>>,
}

impl <P: SimEventPayload> Stream for TickStream<P> {
    type Item = Vec<SimEvent<P>>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut shared = self.shared.lock().unwrap();
        shared.woke = false;

        if let Some(events) = shared.advance_and_fetch() {
            Poll::Ready(Some(events))
        }else{
            if shared.waker.is_none() {
                self.waker = Some(cx.waker().clone());
            }
            Poll::Pending
        }
    }
}

struct SimulatorInternal<P: SimEventPayload> {
    clock: u64,
    pub speed_of_light: u64,
    queue: Vec<SimEvent<P>>,
    woke: bool,
    waker: Option<Waker>,
}

impl <P: SimEventPayload> SimulatorInternal<P> {
    fn advance_and_fetch (&self) -> Option<Vec<SimEvent<P>>> {
        let mut shared = self.shared.lock().unwrap();

        if shared.queue.length == 0 {
            return vec![]
        }

        let next_tick = shared.queue[0].dest_point.t;

        shared.clock = next_tick;
        let split_index = partition(&mut shared.queue, |evt| evt.dest_point.t <= next_tick );

        debug!(%split_index);

        if split_index > 0 {
            Some(shared.queue.drain(0..split_index).collect())
        }else{
            None
        }
    }
}


#[derive(PartialEq, Debug)]
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

pub struct SimEvent<SimEventPayload> {
    pub source_point: Point4,
    pub dest_point: Point4,
    pub payload: SimEventPayload
}

impl <P: SimEventPayload> SimEvent<P> {
//    pub fn fire (self) -> Box<dyn Future<Output=()>>{
//        self.payload.fire()
//    }
}

impl <P: std::fmt::Debug> fmt::Debug for SimEvent<P>{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SimEvent")
        .field("t", &self.dest_point.t )
        .field("payload", &self.payload)
        .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug)]
    struct DummyPayload {}
    impl SimEventPayload for DummyPayload {
        fn fire(self) -> Box<dyn Future> {
            future::ready()
        }
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

        let dests : Vec<Point4>= sim.advance_and_fetch().into_iter().map(|e| e.dest_point ).collect();
        // NOTE: at present, we are reversing the event order for identical timeslots.
        // In theory this shouldn't matter, as no communications should arrive in less than one clock tick
        assert_eq!( dests , vec![Point4 { x: -1, y: 0, z: 0, t: 1 }, Point4 { x: 0, y: 1, z: 0, t: 1 }, Point4 { x: 1, y: 0, z: 0, t: 1 }]);

        let dests : Vec<Point4>= sim.advance_and_fetch().into_iter().map(|e| e.dest_point ).collect();
        assert_eq!( dests , vec![Point4 { x: 2, y: 0, z: 0, t: 2 }] );

        let dests : Vec<Point4>= sim.advance_and_fetch().into_iter().map(|e| e.dest_point ).collect();
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