extern crate unbase;

use std::{thread, time};

//#[test]
fn init_blackhole() {
    let net = unbase::Network::create_new_system();
    let blachole = unbase::network::transport::Blackhole::new();
    net.add_transport( Box::new(blachole) );
    {
        let slab_a = unbase::Slab::new(&net);
        let context_a = slab_a.create_context();
    }

    // Slabs should have been dropped by now
    assert!( net.get_all_local_slabs().len() == 0 );
}

//#[test]
fn init_local_single() {
    let net = unbase::Network::create_new_system();
    {
        let slab_a = unbase::Slab::new(&net);
        let context_a = slab_a.create_context();
    }

    // Slabs should have been dropped by now
    assert!( net.get_all_local_slabs().len() == 0 );
}

//#[test]
fn init_local_multi() {

    let net = unbase::Network::create_new_system();
    {
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

        // NOTE: commenting out the contexts here breaks the refcount cycle, and the slabs are able
        // to go out of scope. With them as is, the slabs do not

        //slab -> network -> transports -> thread list -> closures? -> ??
        //                                                         was strong slab, now weak (local_direct)
        let _context_a = slab_a.create_context();
        let _context_b = slab_b.create_context();
        let _context_c = slab_c.create_context();
    }

    // We should have zero slabs resident at this point
    assert!( net.get_all_local_slabs().len() == 0 );
}

#[test]
fn init_udp() {

    let t1 = thread::spawn(|| {
     {
        let net1 = unbase::Network::create_new_system();
        {
            let udp1 = unbase::network::transport::TransportUDP::new("127.0.0.1:12345".to_string());
            net1.add_transport( Box::new(udp1.clone()) );
            let slab_a = unbase::Slab::new(&net1);
            println!("MARK 1.0");
            thread::sleep( time::Duration::from_millis(150) );
            println!("MARK 1.1");
            assert_eq!( slab_a.peer_slab_count(), 1 );
            println!("MARK 1.2");
        }
        println!("MARK 1.2");

        // my local slab should have dropped
        assert_eq!( net1.get_all_local_slabs().len(), 0 );

        println!("MARK 1.3");
    }
    println!("MARK 1.4");

    });

    thread::sleep( time::Duration::from_millis(50) );

    let t2 = thread::spawn(|| {
        {
            let net2 = unbase::Network::new();
            net2.hack_set_next_slab_id(200);

            {
                let udp2 = unbase::network::transport::TransportUDP::new("127.0.0.1:1337".to_string());
                net2.add_transport( Box::new(udp2.clone()) );

                let slab_b = unbase::Slab::new(&net2);

                udp2.seed_address_from_string( "127.0.0.1:12345".to_string() );
                thread::sleep( time::Duration::from_millis(50) );

                println!("MARK 3.0");
                assert_eq!( slab_b.peer_slab_count(), 1 );
                println!("MARK 3.1");
            }
            println!("MARK 4");

            assert_eq!( net2.get_all_local_slabs().len(), 0 );
            println!("MARK 5");
        }
        println!("MARK 6");

    });

    println!("MARK 7");
    t1.join().expect("thread1.join");
    println!("MARK 8");
    t2.join().expect("thread2.join");
    println!("MARK 9");
}

//#[test]
fn avoid_unnecessary_chatter() {

    let net = unbase::Network::create_new_system();
    {
        let slab_a = unbase::Slab::new(&net);
        let slab_b = unbase::Slab::new(&net);

        let _context_a = slab_a.create_context();
        let _context_b = slab_b.create_context();

        thread::sleep(time::Duration::from_millis(100));

        println!("Slab A MemoRefs present {}", slab_a.count_of_memorefs_resident() );
        println!("Slab A MemoRefs present {}", slab_b.count_of_memorefs_resident() );

        println!("Slab A Memos received {}", slab_a.count_of_memos_received() );
        println!("Slab B Memos received {}", slab_a.count_of_memos_received() );

        assert!( slab_a.count_of_memos_reduntantly_received() == 0, "Redundant memos received" );
        assert!( slab_b.count_of_memos_reduntantly_received() == 0, "Redundant memos received" );
    }

    assert!( net.get_all_local_slabs().len() == 0 );
}

/*
#[test]
fn many_threads() {
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
