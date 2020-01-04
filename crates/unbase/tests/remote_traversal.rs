#![feature(async_closure)]

extern crate unbase;
use timer::Delay;
use unbase::subject::Subject;
use std::time::Duration;
use futures_await_test::async_test;
use futures::{
    FutureExt,
    pin_mut,
    select,
};
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
    let started = simulator.start();
    debug!(%started);

    let slab_a = unbase::Slab::new(&net);
    let slab_b = unbase::Slab::new(&net);

    let context_a = slab_a.create_context();
    let _context_b = slab_b.create_context();

    let rec_a1 = Subject::new_kv(&context_a, "animal_sound", "Moo").await.unwrap();

    rec_a1.set_value("animal_sound", "Woof").await;

    rec_a1.set_value("animal_sound", "Meow").await;

    simulator.quiescence().await;

    let memo_ids = rec_a1.get_all_memo_ids().await;
    slab_a.remotize_memos(&memo_ids).expect("failed to remotize memos");

    let value = rec_a1.get_value("animal_sound").await;
    assert_eq!(value, Some("Meow".to_string()));


    // TODO NEXT: Ensure that all Memo Sends are completed before the simulator Tick ends, AND
    // All Memo Receives are completed before a read is permitted.
    //
    // QUESTION: How do we actually achieve this? Right now we are ignorant of what memos must
    // be retrieved until the recursion completes. Maybe an optimistic execution is called for,
    // so that we synchronously run everything we can at the time of delivery, but send a retrieval
    // request and yeild to the background acceptor to wait for the response.
    // That way the memo requests triggered by each delivery are "instantaneous" (and thus deterministic)
    //
    // I *THINK* the key is to eliminate the stream entirely, and instead have each delivery phase fully process
    // each message until it either completes, or sends a message and hits a yeild point.
    // But this means that the deliver future must resolve at the yield point, and hand the
    // remaining processing to the receiver of the requested memo, which - CRUCIALLY - will do the same as the above

    // This should make the whole process fully deterministic with the simulator

    // Most likely, the key to the performance tractability of the whole data model is tied up with how efficiently
    // we can execute this optimistic executional model on receive.


    // NOTE: Weeeeird idea to think about: What if we didn't have any timeouts based on real Duration,
    // but rather ticks of the beacon clock? This might be a little bit annoying for the users when
    // they're really isolated from any other parts of the network, but it may have other interesting
    // properties to think about. (Followup question: how are beacon clock ping emission probabilities
    // calculated if not themselves with a Duration?)
    // TODO: Set up a KB to track this kind of question

    // **HACK** - creation of the deferred Context application task solved the deadlock, but resulted in nondeterminism
    Delay::new(Duration::from_millis(100)).await;

    simulator.quiesce_and_stop().await;

    assert_eq!( simulator.get_sent().unwrap(), 48 );
    assert_eq!( simulator.get_delivered().unwrap(), 48 );

    // TODO NEXT - This should be deterministic!
    // assert_eq!( simulator.get_clock().unwrap(), 11 );
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

    rec_a1.set_value("animal_sound","Woof").await;
    rec_a1.set_value("animal_sound","Meow").await;

    // TODO - provide a deterministic way to wait for quiescence when not using the simulator
//    simulator.quiescence().await;
    Delay::new(Duration::from_millis(50)).await;

    slab_a.remotize_memos( &rec_a1.get_all_memo_ids().await ).expect("failed to remotize memos");

    Delay::new(Duration::from_millis(10)).await;

    let value = rec_a1.get_value("animal_sound").await.unwrap();
    assert_eq!(value,   "Meow");

}

#[async_test]
async fn remote_traversal_nondeterministic_udp() {

    let f1 = async move || {
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
        rec_a1.set_value("animal_sound", "Woof").await;
        rec_a1.set_value("animal_sound", "Meow").await;

        // Wait until it's been replicated
//        simulator.quiescence().await;
        Delay::new(Duration::from_millis(50)).await;

        // manually remove the memos
        slab_a.remotize_memos(&rec_a1.get_all_memo_ids().await).expect("failed to remotize memos");

        // Not really any strong reason to wait here, except just to play nice and make sure slab_b's peering is updated
        // TODO: test memo expungement/de-peering, followed immediately by MemoRequest for same
        Delay::new(Duration::from_millis(50)).await;

        // now lets see if we can project rec_a1 animal_sound. This will require memo retrieval from slab_b
        assert_eq!(rec_a1.get_value("animal_sound").await.unwrap(), "Meow");

        Delay::new(Duration::from_millis(500)).await;

        // slab_a drops and goes away
    };


    let f2 = async move || {
        let net2 = unbase::Network::new();
        net2.hack_set_next_slab_id(200);

        // Ensure slab_a is listening - TODO make this auto-retry
        Delay::new(Duration::from_millis(50)).await;

        let udp2 = unbase::network::transport::TransportUDP::new("127.0.0.1:12002".to_string());
        net2.add_transport(Box::new(udp2.clone()));

        let slab_b = unbase::Slab::new(&net2);
        udp2.seed_address_from_string("127.0.0.1:12001".to_string());

        Delay::new(Duration::from_millis(50)).await;
        let _context_b = slab_b.create_context();
        // hang out to keep stuff in scope, and hold off calling the destructors
        // necessary in order to be online so we can answer slab_a's inquiries
        Delay::new(Duration::from_millis(1500)).await;
    };

    let t1 = f1().fuse();
    let t2 = f2().fuse();

    pin_mut!(t1, t2);

    loop {
        select! {
            () = t1 => println!("task one completed"),
            () = t2 => println!("task two completed"),
            complete => break
        }
    }

}
