extern crate unbase;
use unbase::subject::Subject;
use unbase::error::*;

#[test]
fn basic_eventual() {

    let net = unbase::Network::new();
    let simulator = unbase::network::transport::Simulator::new();
    net.add_transport( Box::new(simulator.clone()) );

    let slab_a = unbase::Slab::new(&net);
    let slab_b = unbase::Slab::new(&net);
    let slab_c = unbase::Slab::new(&net);

    assert!(slab_a.id == 0, "Slab A ID shoud be 0");
    assert!(slab_b.id == 1, "Slab B ID shoud be 1");
    assert!(slab_c.id == 2, "Slab C ID shoud be 2");


    assert!(slab_a.peer_slab_count() == 2, "Slab A Should know two peers" );
    assert!(slab_b.peer_slab_count() == 2, "Slab B Should know two peers" );
    assert!(slab_c.peer_slab_count() == 2, "Slab C Should know two peers" );


    let context_a = slab_a.create_context();
    let context_b = slab_b.create_context();
    let context_c = slab_c.create_context();

    let rec_a1 = Subject::new_kv(&context_a, "animal_sound", "Moo");

    assert!(rec_a1.is_ok(), "New subject should be created");
    let rec_a1 = rec_a1.unwrap();

    assert!(rec_a1.get_value("animal_sound").unwrap() == "Moo", "New subject should be internally consistent");

    assert!(slab_a.count_of_memorefs_resident() == 1, "Slab A should have 1 memorefs resident");
    assert!(slab_b.count_of_memorefs_resident() == 0, "Slab B should have 0 memorefs resident");
    assert!(slab_c.count_of_memorefs_resident() == 0, "Slab C should have 0 memorefs resident");

    assert!(context_b.get_subject( rec_a1.id ).unwrap_err() == RetrieveError::NotFound, "new subject should not yet have conveyed to slab B");
    assert!(context_c.get_subject( rec_a1.id ).unwrap_err() == RetrieveError::NotFound, "new subject should not yet have conveyed to slab C");

    simulator.advance_clock(1); // advance the simulator clock by one tick

    assert!(slab_a.count_of_memorefs_resident() == 1, "Slab A should have 1 memorefs resident");
    assert!(slab_b.count_of_memorefs_resident() == 1, "Slab B should have 1 memorefs resident");
    assert!(slab_c.count_of_memorefs_resident() == 1, "Slab C should have 1 memorefs resident");

    let rec_b1 = context_b.get_subject( rec_a1.id );
    let rec_c1 = context_c.get_subject( rec_a1.id );

    assert!(rec_b1.is_ok(), "new subject should now have conveyed to slab B");
    assert!(rec_c1.is_ok(), "new subject should now have conveyed to slab C");

    let rec_b1 = rec_b1.unwrap();
    let rec_c1 = rec_c1.unwrap();

    assert!(rec_b1.get_value("animal_sound").unwrap() == "Moo", "Subject read from Slab B should be internally consistent");
    assert!(rec_c1.get_value("animal_sound").unwrap() == "Moo", "Subject read from Slab C should be internally consistent");

    simulator.advance_clock(1); // advance the simulator clock by one tick

    assert_eq!(rec_a1.get_value("animal_sound").unwrap(), "Moo");
    assert_eq!(rec_b1.get_value("animal_sound").unwrap(), "Moo");
    assert_eq!(rec_c1.get_value("animal_sound").unwrap(), "Moo");


    rec_b1.set_kv("animal_type","Bovine");
    assert_eq!(rec_b1.get_value("animal_type").unwrap(), "Bovine");
    assert_eq!(rec_b1.get_value("animal_sound").unwrap(),   "Moo");

    rec_b1.set_kv("animal_sound","Woof");
    rec_b1.set_kv("animal_type","Kanine");
    assert_eq!(rec_b1.get_value("animal_sound").unwrap(), "Woof");
    assert_eq!(rec_b1.get_value("animal_type").unwrap(),  "Kanine");

    // Should not yet have propagated to slab A
    assert_eq!(rec_a1.get_value("animal_sound").unwrap(),   "Moo");
    assert!(rec_a1.get_value("animal_type").is_none(), "Should not yet have a value on Slab A for animal_type");

    simulator.advance_clock(1); // advance the simulator clock by one tick

    // Nowwww it should have propagated
    assert_eq!(rec_a1.get_value("animal_sound").unwrap(),   "Woof");
    assert_eq!(rec_a1.get_value("animal_type").unwrap(),    "Kanine");

}
