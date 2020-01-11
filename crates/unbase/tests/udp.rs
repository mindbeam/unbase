#![feature(async_closure)]

extern crate unbase;

use timer::Delay;
use std::time::Duration;
use futures_await_test::async_test;
use futures::{
    future::{
        FutureExt
    },
    join,
};
use tracing::info;

#[async_test]
async fn test_udp() {
    unbase_test_util::init_test_logger();

    let t1 = node_a().fuse();
    let t2 = node_b().fuse();

    join!{ t1, t2 };
}


async fn node_a() {
    let net1 = unbase::Network::create_new_system();
    let udp1 = unbase::network::transport::TransportUDP::new("127.0.0.1:12345".to_string());
    net1.add_transport(Box::new(udp1.clone()));
    let _slab_a = unbase::Slab::new(&net1);
    Delay::new(Duration::from_millis(500)).await;

    info!("Node A is done!");
}

async fn node_b() {
    Delay::new(Duration::from_millis(50)).await;

    let net2 = unbase::Network::new();
    net2.hack_set_next_slab_id(200);

    let udp2 = unbase::network::transport::TransportUDP::new("127.0.0.1:1337".to_string());
    net2.add_transport(Box::new(udp2.clone()));

    let _slab_b = unbase::Slab::new(&net2);

    udp2.seed_address_from_string("127.0.0.1:12345".to_string());
    Delay::new(Duration::from_millis(500)).await;

    info!("Node B is done!");
    // TODO improve this test to actually exchange something, or at least verify that we've retrieved the root index
}