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
        pin::Pin,
        sync::{Arc, Mutex},
        task::{Context, Poll, Waker},
    },
};

use async_trait::async_trait;
use tracing::{span,Level};
use itertools::partition;
use tracing::debug;

#[derive(Debug)]
pub enum SimError{
    /// Operation was aborted because of the potential for nondeterminism
    Nondeterminism
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

pub struct SimEventItem<E: SimEvent> {
    pub source: Point4,
    pub destination: Point4,
    pub event: E
}

//impl <P: SimEventPayload> SimEventItem<P> {
//    pub fn fire (self) -> Box<dyn Future<Output=()>>{
//        self.payload.fire()
//    }
//}

impl <E: SimEvent + std::fmt::Debug> fmt::Debug for SimEventItem<E>{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SimEvent")
            .field("t", &self.destination.t )
            .field("payload", &self.event)
            .finish()
    }
}

#[async_trait]
pub trait SimEvent {
    async fn fire(self);
}

pub struct Simulator<E: SimEvent> {
    shared: Arc<Mutex<SimulatorInternal<E>>>,
    runner: Arc<Mutex<Option<RemoteHandle<()>>>>,
}

impl <E: SimEvent> Clone for Simulator<E>{
    fn clone(&self) -> Self {
        Self{
            shared: self.shared.clone(),
            runner: self.runner.clone(),
        }
    }
}

impl <E: SimEvent + 'static + Send + fmt::Debug> Simulator<E> {
    pub fn new() -> Self {
        let sim = Simulator {
            shared: Arc::new(Mutex::new(
                SimulatorInternal::<E> {
                    clock: 0,
                    queue: Vec::new(),
                    woke: false,
                    waker: None,
                }
            )),
            runner: Arc::new(Mutex::new(None))
        };

//        sim.start();

        sim
    }
    /// If the simulator is currentlly applying events, no new events can be observed until the next tick
    // In the case that we want to model computational time as being more than 1 tick, we should queue the events with a different departure time
    pub fn add_event(&self, event: E, source: &Point3, destination: &Point3 ) {

        let mut shared = self.shared.lock().unwrap();

        let xsq = (source.x - destination.x).abs().pow(2);
        let ysq = (source.y - destination.y).abs().pow(2);
        let zsq = (source.z - destination.z).abs().pow(2);

        // QUESTION: should we allow fractional distances?
        // If so, will it require fractional clock ticks?
        // do we just tick forward to the nearest whole number and use the non-whole part for ordering?
        let mut distance = ((xsq + ysq + zsq) as f64).sqrt().ceil() as u64;

        if distance == 0 {
            distance = 1;
        }

        let source = Point4 {
            x: source.x,
            y: source.y,
            z: source.z,
            t: shared.clock
        };

        let destination = Point4 {
            x: destination.x,
            y: destination.y,
            z: destination.z,
            t: shared.clock + distance
        };

        let seek = destination.t;
        let idx = shared.queue.binary_search_by(|probe| probe.destination.t.cmp(&seek)).unwrap_or_else(|x| x);
        shared.queue.insert(idx, SimEventItem{
            event,
            source,
            destination,
        });

        if let Some(ref waker) = shared.waker {
            // debounce
            if !shared.woke {
                waker.wake_by_ref();
                shared.woke = true;
            }
        }
    }
    pub fn get_clock(&self) -> Result<u64,SimError> {
        match *self.runner.lock().unwrap() {
            Some(_) => Err(SimError::Nondeterminism),
            None => {
                Ok(self.shared.lock().unwrap().clock)
            }
        }
    }
    pub fn get_clock_nondeterministic(&self) -> u64 {
        self.shared.lock().unwrap().clock
    }

    #[tracing::instrument(level = "debug")]
    pub async fn advance_clock (&self) -> Result<Option<u64>, SimError>{

        if let Some(_) = *self.runner.lock().unwrap() {
            // Can't manually advance the clock while the background runner is running
            return Err(SimError::Nondeterminism);
        }

        let tick = {
            let mut shared = self.shared.lock().unwrap();
            shared.woke = false;
            shared.advance_and_fetch()
        };

        // Gotta fire the events outside of the shared lock, because they might call add_events

        if let Some((clock,events)) = tick {
            stream::iter(events).for_each_concurrent(
                None,
                |rx| async move {
                    rx.event.fire().await
                }
            ).await;
            return Ok(Some(clock))
        }else{
            Ok(None)
        }
    }
    fn tickstream (&self) -> TickStream<E> {
        // TODO: store the tickstream and/or prevent there from being two?
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
                        rx.event.fire().await
                    }
                ).await;

                // TODO: consider adding a timeout here to check if the simulator might be logjammed
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

struct TickStream<P: SimEvent> {
    shared: Arc<Mutex<SimulatorInternal<P>>>,
}

impl <P: SimEvent> Stream for TickStream<P> {
    type Item = Vec<SimEventItem<P>>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut shared = self.shared.lock().unwrap();
        shared.woke = false;

        if let Some((_clock,events)) = shared.advance_and_fetch() {
            Poll::Ready(Some(events))
        }else{
            if shared.waker.is_none() {
                shared.waker = Some(cx.waker().clone());
            }
            Poll::Pending
        }
    }
}

struct SimulatorInternal<P: SimEvent> {
    clock: u64,
    queue: Vec<SimEventItem<P>>,
    woke: bool,
    waker: Option<Waker>,
}

impl <P: SimEvent> SimulatorInternal<P> {
    fn advance_and_fetch (&mut self) -> Option<(u64, Vec<SimEventItem<P>>)> {

        if self.queue.len() == 0 {
            return None
        }

        // The queue should be ordered by arrival time from add_event
        let next_tick = self.queue[0].destination.t;

        self.clock = next_tick;
        let split_index = partition(&mut self.queue, |item| item.destination.t <= next_tick );

        debug!(%split_index);

        if split_index > 0 {
            Some((self.clock, self.queue.drain(0..split_index).collect()))
        }else{
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures_await_test::async_test;
    use timer::Delay;

    #[derive(Debug)]
    struct DummyPayload {}

    #[async_trait]
    impl SimEvent for DummyPayload {
        async fn fire(self) {
            //
        }
    }

    #[test]
    fn delivery_order() {
        // repeat 10 times due to check for nondeterminism
        for i in 0..10 {
            debug!(%i);
            let sim = Simulator::<DummyPayload>::new();

            sim.add_event(
                DummyPayload {},
                &Point3 { x: 0, y: 0, z: 0 },
                &Point3 { x: 10, y: 0, z: 0 },
            );
            sim.add_event(
                DummyPayload {},
                &Point3 { x: 0, y: 0, z: 0 },
                &Point3 { x: -10, y: 0, z: 0 },
            );
            sim.add_event(
                DummyPayload {},
                &Point3 { x: 0, y: 0, z: 0 },
                &Point3 { x: 30, y: 0, z: 0 },
            );
            sim.add_event(
                DummyPayload {},
                &Point3 { x: 0, y: 0, z: 0 },
                &Point3 { x: 0, y: 10, z: 0 },
            );
            sim.add_event(
                DummyPayload {},
                &Point3 { x: 0, y: 0, z: 0 },
                &Point3 { x: 20, y: 0, z: 0 },
            );
            sim.add_event(
                DummyPayload {},
                &Point3 { x: 0, y: 0, z: 0 },
                &Point3 { x: 11, y: 10, z: 0 },
            );

            let seq: Vec<(u64,u64)> = sim.shared.lock().unwrap().queue.iter().map(|e| (e.source.t, e.destination.t)).collect();
            assert_eq!(seq, vec![(0u64, 10u64), (0,10), (0,10),(0,15), (0,20), (0,30)]);

            let (_clock,events) = {
                sim.shared.lock().unwrap().advance_and_fetch().unwrap()
            };
            let dests = events.into_iter().map(|e| e.destination).collect::<Vec<Point4>>();

            // NOTE: at present, we are reversing the event order for identical timeslots.
            // In theory this shouldn't matter, as no communications should arrive in less than one clock tick
            assert_eq!(dests, vec![Point4 { x: -10, y: 0, z: 0, t: 10 }, Point4 { x: 0, y: 10, z: 0, t: 10 }, Point4 { x: 10, y: 0, z: 0, t: 10 }]);

            let (_clock,events) = {
                sim.shared.lock().unwrap().advance_and_fetch().unwrap()
            };
            let dests = events.into_iter().map(|e| e.destination).collect::<Vec<Point4>>();

            assert_eq!(dests, vec![Point4 { x: 11, y: 10, z: 0, t: 15 }]);

            let (_clock,events) = {
                sim.shared.lock().unwrap().advance_and_fetch().unwrap()
            };
            let dests = events.into_iter().map(|e| e.destination).collect::<Vec<Point4>>();

            assert_eq!(dests, vec![Point4 { x: 20, y: 0, z: 0, t: 20 }]);

            let (_clock,events) = {
                sim.shared.lock().unwrap().advance_and_fetch().unwrap()
            };
            let dests = events.into_iter().map(|e| e.destination).collect::<Vec<Point4>>();

            assert_eq!(dests, vec![Point4 { x: 30, y: 0, z: 0, t: 30 }]);
        }
    }

    #[derive(Debug)]
    struct EventIssuerPayload {
        sim: Simulator<Self>,
        generation: u32,
        max: u32,
        fanout: u32,
    }

    #[async_trait]
    impl SimEvent for EventIssuerPayload {
        async fn fire(self) {

            if self.generation < self.max {
                let generation = self.generation + 1;
                for _ in 0..self.fanout {
                    let sim = self.sim.clone();
                    self.sim.add_event(
                        EventIssuerPayload {
                            sim,
                            generation,
                            max: self.max,
                            fanout: self.fanout,
                        },
                        &Point3 { x: 0, y: 0, z: 0 },
                        &Point3 { x: ((10 * generation) % 7) as i64, y: ((10 * generation) % 9) as i64, z: ((10 * generation) % 11) as i64 },
                    );
                }
            }
        }
    }

    #[async_test]
    async fn stream_nofanout (){
        let sim = Simulator::<EventIssuerPayload>::new();

        assert_eq!( sim.get_clock().unwrap(), 0 );

        sim.start();

        sim.add_event(
            EventIssuerPayload{ sim: sim.clone(), generation: 0, max: 6, fanout: 1 },
            &Point3 { x: 0, y: 0, z: 0 },
            &Point3 { x: 10, y: 0, z: 0 },
        );

        // TODO: replace this with sim.quiesce()
        use std::time::Duration;

        Delay::new(Duration::from_millis(1000)).await;

        sim.stop();
        assert_eq!( sim.get_clock().unwrap(), 68);
    }

    // TODO: BROKEN - goes into a busy loop
    //#[async_test]
    async fn stream_fanout (){
        let sim = Simulator::<EventIssuerPayload>::new();

        assert_eq!( sim.get_clock().unwrap(), 0 );

        sim.start();

        sim.add_event(
            EventIssuerPayload{ sim: sim.clone(), generation: 0, max: 1, fanout: 3 },
            &Point3 { x: 0, y: 0, z: 0 },
            &Point3 { x: 10, y: 0, z: 0 },
        );

        // TODO: replace this with sim.quiesce()
        use std::time::Duration;
        Delay::new(Duration::from_millis(1000)).await;

        sim.stop();
        assert_eq!( sim.get_clock().unwrap(), 68);
    }
}

impl <P: SimEvent + fmt::Debug> fmt::Debug for Simulator<P> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let shared = self.shared.lock().unwrap();
        fmt.debug_struct("Simulator")
            .field("clock",&shared.clock)
            .field("queue", &shared.queue)
            .finish()
    }
}