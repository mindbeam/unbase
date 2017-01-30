extern crate unbase;
//use std::thread;

#[test]
fn test_init() {
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

    let _context_a = slab_a.create_context();
    let _context_b = slab_b.create_context();
    let _context_c = slab_c.create_context();

}

/*
#[test]
fn test_meow(){

        println!("Slab1 Before: {:?}", &slab1);
        println!("Slab2 Before: {:?}", &slab2);
        //println!("Resident Before: {}", slab2.count_of_memos_resident());
        net.deliver_all_memos();

        println!("Resident After: {}", slab2.count_of_memos_resident());
        assert!(slab2.count_of_memos_resident() == 2, "Memos resident should be 2");
}
*/

// TODO: update internals to allow deliver_memos to fully deliver them to the slabs
// in question. This is necessary for deterministic testing. Will entail some
// rethinking of per-slab channels. Will have to balance the need for deterministic
// test cases vs concurrency in a production scenario.


/*
#[test]
fn test_threads() {
    let net = unbase::Network::new();

    let mut threads = Vec::new();
    for _ in 0..20 {
        let net = net.clone();

        threads.push(thread::spawn(move || {
            let slab = unbase::Slab::new(&net);
            assert!(slab.id > 0, "Nonzero Slab ID");
            println!("info test thread. Slab: {}", slab.id);
        }));
    }

    for t in threads {
        t.join().unwrap();
    }

    // println!("{:?}", net);
}
*/
