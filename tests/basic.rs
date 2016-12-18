extern crate unbase;
use std::thread;

#[test]
fn test_init() {
    let net = unbase::Network::new();

    let slab1 = unbase::Slab::new(&net);
    let slab2 = unbase::Slab::new(&net);

    assert!(slab1.id == 1, "Slab 1 ID shoud be 1");
    assert!(slab2.id == 2, "Slab 2 ID shoud be 2");

    // TODO: update internals to allow deliver_memos to fully deliver them to the slabs
    // in question. This is necessary for deterministic testing. Will entail some
    // rethinking of per-slab channels. Will have to balance the need for deterministic
    // test cases vs concurrency in a production scenario.

    net.deliver_all_memos();

    println!("Resident: {}", slab2.count_of_memos_resident());
    assert!(slab2.count_of_memos_resident() == 2, "Memos resident should be 2");

}

#[test]
fn test_threads(){
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

    //println!("{:?}", net);
}
