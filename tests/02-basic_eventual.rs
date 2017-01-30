extern crate unbase;
use unbase::subject::Subject;

#[test]
fn basic_eventual() {
    let net = unbase::Network::new();

    let slab_a = unbase::Slab::new(&net);
    let slab_b = unbase::Slab::new(&net);
    let slab_c = unbase::Slab::new(&net);

    assert!(slab_a.id == 1, "Slab A ID shoud be 1");
    assert!(slab_b.id == 2, "Slab B ID shoud be 2");
    assert!(slab_c.id == 3, "Slab C ID shoud be 3");


    assert!(slab_a.peer_slab_count() == 2, "Slab A Should know two peers" );
    assert!(slab_b.peer_slab_count() == 2, "Slab B Should know two peers" );
    assert!(slab_c.peer_slab_count() == 2, "Slab C Should know two peers" );


    let context_a = slab_a.create_context();
    //let context_b = slab_b.create_context();
    //let _context_c = slab_c.create_context();

    let rec_a1 = Subject::new_kv(context_a, "animal_sound", "Moo");

    assert!(rec_a1.is_ok(), "New subject should be created");
    let rec_a1 = rec_a1.unwrap();

    assert!(rec_a1.get_value("animal_sound").unwrap() == "Moo", "New subject should be internally consistent");

    /*

    assert!(context_b.get_subject( rec_a1.id ).is_ok(), "new subject should not yet have conveyed to slab B");


    // Time moves forward
    net.deliver_all_memos();

    let rec_b1 = context_b.get_subject( rec_a1.id );
    assert!(rec_b1.is_ok(), "new subject should now be available on slab B");
    let rec_b1 = rec_b1.unwrap();

    assert!(rec_b1.get_value("animal_sound").unwrap() == "moo", "Transferred subject should be consistent");


    // Time moves forward
    net.deliver_all_memos();

    let rec_c1 = context_c.get_subject( rec_a1.id );
    assert!(rec_c1.is_ok(), "new subject should now be available on slab C");
    let mut rec_c1 = rec_c1.unwrap();

    assert!(rec_c1.get_value("animal_sound").unwrap() == "moo", "Transferred subject should be consistent");

    // Time moves forward
    net.deliver_all_memos();

    assert!( rec_c1.set_kv("animal_sound", "woof"), "Change the value on slab C" );
    assert!( rec_c1.get_value("animal_sound").unwrap() == "woof", "Updated subject should be consistent");

    assert!( rec_a1.get_value("animal_sound").unwrap() == "moo", "Value should be unchanged on slab A" );

    // Time moves forward
    net.deliver_all_memos();

    assert!( rec_a1.get_value("animal_sound").unwrap() == "woof", "Now the value should be changed on slab A" );
*/

}
