#![feature(async_closure)]

extern crate unbase;
use timer::Delay;
use unbase::subject::Subject;
use std::time::Duration;
use async_std::task::block_on;
use futures_await_test::async_test;
use futures::future::{select, RemoteHandle};
use tracing::{debug, span, Level};

#[async_test]
async fn remote_traversal_simulated() {

    // TODO init logging and add span in async_test macro
    unbase_test_util::init_test_logger("remote_traversal");
    let testspan = span!(Level::INFO, "remote_traversal_simulated");
    let _enter = testspan.enter();

    let net = unbase::Network::create_new_system();
    let simulator = unbase::util::simulator::Simulator::new();
    net.add_transport(Box::new(simulator.clone()));

    let slab_a = unbase::Slab::new(&net);
    let slab_b = unbase::Slab::new(&net);

    let context_a = slab_a.create_context();
    let _context_b = slab_b.create_context();

    let rec_a1 = Subject::new_kv(&context_a, "animal_sound", "Moo").await.unwrap();

    rec_a1.set_value("animal_sound", "Woof");
    rec_a1.set_value("animal_sound", "Meow");

    simulator.advance_clock(1); // Now it should have propagated to slab B

    simulator.advance_clock(1); // now slab A should know that Slab B has it
    simulator.advance_clock(1);
    simulator.advance_clock(1);
    simulator.advance_clock(1);

    slab_a.remotize_memo_ids(&rec_a1.get_all_memo_ids()).expect("failed to remotize memos");
//
    simulator.start();
//
    // This should be reconsidered when the simulator is reworked per https://github.com/unbase/unbase/issues/6
//

        let value = rec_a1.get_value("animal_sound").await.expect("get_value");
//
//    let s2 = simulator.clone();
//    let t = std::thread::spawn(move || {
//        for _ in 0..300 {
//            debug!("ADVANCE");
//            block_on(Delay::new(Duration::from_millis(10)));
//            s2.advance_clock(1);
//        }
//    });
//
//    handle.await;
//    t.join().unwrap();
}

#[async_test]
async fn remote_traversal_nondeterministic() {

    let net = unbase::Network::create_new_system();
    // Automatically uses LocalDirect, which should be much faster than the simulator, but is also nondeterministic.
    // This will be used in production for slabs that cohabitate the same process

    let slab_a = unbase::Slab::new(&net);
    let slab_b = unbase::Slab::new(&net);

    let context_a = slab_a.create_context();
    let _context_b = slab_b.create_context();

    let rec_a1 = Subject::new_kv(&context_a, "animal_sound", "Moo").await.unwrap();

    rec_a1.set_value("animal_sound","Woof");
    rec_a1.set_value("animal_sound","Meow");

    Delay::new(Duration::from_millis(10)).await;

    slab_a.remotize_memo_ids( &rec_a1.get_all_memo_ids() ).expect("failed to remotize memos");

    Delay::new(Duration::from_millis(10)).await;

    let value = rec_a1.get_value("animal_sound").await.unwrap();
    assert_eq!(value,   "Meow");

}

#[async_test]
async fn remote_traversal_nondeterministic_udp() {

    let h1: RemoteHandle<()> = unbase::util::task::spawn_with_handle((async move || {
        let net1 = unbase::Network::create_new_system();

        let udp1 = unbase::network::transport::TransportUDP::new("127.0.0.1:12001".to_string());
        net1.add_transport(Box::new(udp1.clone()));
        let slab_a = unbase::Slab::new(&net1);

        // no reason to wait to create the context here
        let context_a = slab_a.create_context();

        // wait for slab_b to be on the peer list, and to be hooked in to our root_index_seed
        Delay::new(Duration::from_millis(150)).await;

        // Do some stuff
        let rec_a1 = Subject::new_kv(&context_a, "animal_sound", "Moo").await.unwrap();
        rec_a1.set_value("animal_sound", "Woof");
        rec_a1.set_value("animal_sound", "Meow");

        // Wait until it's been replicated
        Delay::new(Duration::from_millis(150)).await;

        // manually remove the memos
        slab_a.remotize_memo_ids(&rec_a1.get_all_memo_ids()).expect("failed to remotize memos");

        // Not really any strong reason to wait here, except just to play nice and make sure slab_b's peering is updated
        // TODO: test memo expungement/de-peering, followed immediately by MemoRequest for same
        Delay::new(Duration::from_millis(50)).await;

        // now lets see if we can project rec_a1 animal_sound. This will require memo retrieval from slab_b
        assert_eq!(rec_a1.get_value("animal_sound").await.unwrap(), "Meow");

        Delay::new(Duration::from_millis(500)).await;

        // slab_a drops and goes away
    })());

    // Ensure slab_a is listening
    Delay::new(Duration::from_millis(50)).await;

    let h2: RemoteHandle<()> = unbase::util::task::spawn_with_handle((async move || {
        let net2 = unbase::Network::new();
        net2.hack_set_next_slab_id(200);

        let udp2 = unbase::network::transport::TransportUDP::new("127.0.0.1:12002".to_string());
        net2.add_transport(Box::new(udp2.clone()));

        let slab_b = unbase::Slab::new(&net2);
        udp2.seed_address_from_string("127.0.0.1:12001".to_string());

        Delay::new(Duration::from_millis(50)).await;
        let _context_b = slab_b.create_context();
        // hang out to keep stuff in scope, and hold off calling the destructors
        // necessary in order to be online so we can answer slab_a's inquiries
        Delay::new(Duration::from_millis(1500)).await;
    })());

    select(h1, h2).await;

}
