extern crate unbase;

use std::{thread, time};
use futures_await_test::async_test;
use futures::executor::block_on;

#[async_test]
async fn test_udp() {

    let t1 = thread::spawn(|| {
        block_on(async {
            let net1 = unbase::Network::create_new_system();
            let udp1 = unbase::network::transport::TransportUDP::new("127.0.0.1:12345".to_string());
            net1.add_transport(Box::new(udp1.clone()));
            let _slab_a = unbase::Slab::new(&net1);

            //    thread::sleep( time::Duration::from_secs(5) );
        })
    });

    thread::sleep( time::Duration::from_millis(50) );

    let t2 = thread::spawn(|| {
        block_on(async {
            let net2 = unbase::Network::new();
            net2.hack_set_next_slab_id(200);

            let udp2 = unbase::network::transport::TransportUDP::new("127.0.0.1:1337".to_string());
            net2.add_transport(Box::new(udp2.clone()));

            let _slab_b = unbase::Slab::new(&net2);

            udp2.seed_address_from_string("127.0.0.1:12345".to_string());
            thread::sleep(time::Duration::from_millis(500));
        })
    });

    t1.join().expect("thread1.join");
    t2.join().expect("thread2.join");
}
