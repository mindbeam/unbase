#![feature(async_closure)]

extern crate unbase;
use unbase::subject::Subject;
use unbase::error::*;
use futures_await_test::async_test;

use tracing::debug;

#[async_test]
async fn basic_eventual() {
    unbase_test_util::init_test_logger();

    // initialize the system, and use the simulator transport
    let net = unbase::Network::create_new_system();
    let simulator = unbase::util::simulator::Simulator::new();
    net.add_transport( Box::new(simulator.clone()) );

    // Set up three Slabs, corresponding to three hosts, or three OS processes, or MAYBE even three different workers within the same OS process
    let slab_a = unbase::Slab::new(&net);
    let slab_b = unbase::Slab::new(&net);
    let slab_c = unbase::Slab::new(&net);

    // Basic sanity tests
    assert!(slab_a.id == 0, "Slab A ID shoud be 0");
    assert!(slab_b.id == 1, "Slab B ID shoud be 1");
    assert!(slab_c.id == 2, "Slab C ID shoud be 2");

    assert!(slab_a.peer_slab_count() == 2, "Slab A Should know two peers" );
    assert!(slab_b.peer_slab_count() == 2, "Slab B Should know two peers" );
    assert!(slab_c.peer_slab_count() == 2, "Slab C Should know two peers" );

    // Spawns a task to tick away automatically in the background
    simulator.start();

    let context_a = slab_a.create_context();
    let mut context_b = slab_b.create_context();
    let mut context_c = slab_c.create_context();

    let rec_a1 = Subject::new_kv(&context_a, "animal_sound", "Moo").await;
    assert!(rec_a1.is_ok(), "New subject should be created");
    let rec_a1 = rec_a1.unwrap();

    assert!(rec_a1.get_value("animal_sound").await.unwrap() == "Moo", "New subject should be internally consistent");

    // TODO: consolidation is necessary for eventual consistency to work
    //context_a.fully_consolidate();

    debug!("New subject ID {}", rec_a1.id );

    let record_id = rec_a1.id;

    // TODO: move this to another test
    let root_index_subject_id = if let Some(ref s) = *context_a.inner.0.root_index.read().unwrap() {
        s.get_root_id()
    }else{
        panic!("sanity error - uninitialized context");
    };

    assert_eq!(context_b.get_subject_by_id(record_id).await.unwrap_err(), RetrieveError::NotFound, "new subject should not yet have conveyed to slab B");
    assert_eq!(context_c.get_subject_by_id(record_id).await.unwrap_err(), RetrieveError::NotFound, "new subject should not yet have conveyed to slab C");

    simulator.quiesce().await;

    debug!("Root Index = {:?}", context_b.get_subject_head_memo_ids(root_index_subject_id)  );

    // TODO: replace this – Temporary way to magically, instantly send context
    debug!("Manually exchanging context from Context A to Context B - Count of MemoRefs: {}", context_a.hack_send_context(&mut context_b) );
    debug!("Manually exchanging context from Context A to Context C - Count of MemoRefs: {}", context_a.hack_send_context(&mut context_c) );
    debug!("Root Index = {:?}", context_b.get_subject_head_memo_ids(root_index_subject_id)  );

    let rec_b1 = context_b.get_subject_by_id( record_id ).await;
    let rec_c1 = context_c.get_subject_by_id( record_id ).await;

    assert!(rec_b1.is_ok(), "new subject should now have conveyed to slab B");
    assert!(rec_c1.is_ok(), "new subject should now have conveyed to slab C");

    let rec_b1 = rec_b1.unwrap();
    let rec_c1 = rec_c1.unwrap();

    assert!(rec_b1.get_value("animal_sound").await.unwrap() == "Moo", "Subject read from Slab B should be internally consistent");
    assert!(rec_c1.get_value("animal_sound").await.unwrap() == "Moo", "Subject read from Slab C should be internally consistent");

    assert_eq!(rec_a1.get_value("animal_sound").await.unwrap(), "Moo");
    assert_eq!(rec_b1.get_value("animal_sound").await.unwrap(), "Moo");
    assert_eq!(rec_c1.get_value("animal_sound").await.unwrap(), "Moo");

    // Now lets make some changes

    rec_b1.set_value("animal_type","Bovine").await;
    assert_eq!(rec_b1.get_value("animal_type").await.unwrap(), "Bovine");
    assert_eq!(rec_b1.get_value("animal_sound").await.unwrap(),   "Moo");

    rec_b1.set_value("animal_sound","Woof").await;
    rec_b1.set_value("animal_type","Kanine").await;
    assert_eq!(rec_b1.get_value("animal_sound").await.unwrap(), "Woof");
    assert_eq!(rec_b1.get_value("animal_type").await.unwrap(),  "Kanine");

    // Should not yet have propagated to slab A
    assert_eq!(rec_a1.get_value("animal_sound").await.unwrap(),   "Moo");
    assert_eq!(rec_a1.get_value("animal_type").await, None, "Should not yet have a value on Slab A for animal_type");

    simulator.start(); // advance the simulator clock by one tick
    simulator.quiesce().await;

    // Nowwww it should have propagated
    assert_eq!(rec_a1.get_value("animal_sound").await.unwrap(),   "Woof");
    assert_eq!(rec_a1.get_value("animal_type").await.unwrap(),    "Kanine");

    simulator.quiesce_and_stop().await;
}
