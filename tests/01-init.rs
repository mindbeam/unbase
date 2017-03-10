extern crate unbase;
//use std::thread;

#[test]
fn test_init() {
    let simulator = unbase::network::Simulator::new();
    let net = unbase::Network::new(&simulator);

    let slab_a = unbase::Slab::new(&net);
    let slab_b = unbase::Slab::new(&net);
    let slab_c = unbase::Slab::new(&net);

    // make sure the slab is properly initialized. get_ref will panic if not
    let _slabref_a = slab_a.get_ref();
    let _slabref_b = slab_b.get_ref();
    let _slabref_c = slab_c.get_ref();

    assert!(slab_a.id == 0, "Slab A ID shoud be 0");
    assert!(slab_b.id == 1, "Slab B ID shoud be 1");
    assert!(slab_c.id == 2, "Slab C ID shoud be 2");


    assert!(slab_a.peer_slab_count() == 2, "Slab A Should know two peers" );
    assert!(slab_b.peer_slab_count() == 2, "Slab B Should know two peers" );
    assert!(slab_c.peer_slab_count() == 2, "Slab C Should know two peers" );

    let _context_a = slab_a.create_context();
    let _context_b = slab_b.create_context();
    let _context_c = slab_c.create_context();

}

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
            println!("# info test thread. Slab: {}", slab.id);
        }));
    }

    for t in threads {
        t.join().unwrap();
    }

    // println!("# {:?}", net);
}
*/
