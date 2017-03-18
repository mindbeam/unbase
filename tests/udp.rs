extern crate unbase;

#[test]
fn test_udp() {

    let net1 = unbase::Network::new();
    let udp1 = unbase::network::transport::TransportUDP::new("127.0.0.1:12345".to_string());
    net1.add_transport( Box::new(udp1.clone()) );

    let net2 = unbase::Network::new();
    let udp2 = unbase::network::transport::TransportUDP::new("127.0.0.1:1337".to_string());
    net2.add_transport( Box::new(udp2.clone()) );

    udp2.seed_address("127.0.0.1:12345".to_string());

    //use std::{thread, time};
    //thread::sleep( time::Duration::from_secs(50) );
}
