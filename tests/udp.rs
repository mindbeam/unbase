extern crate unbase;

use std::{thread, time};

//#[test]
fn test_udp() {

    let net1 = unbase::Network::new();
    let udp1 = unbase::network::transport::TransportUDP::new("127.0.0.1:12345".to_string());
    net1.add_transport( Box::new(udp1.clone()) );

    let t2 = thread::spawn(|| {
        let net2 = unbase::Network::new();
        let udp2 = unbase::network::transport::TransportUDP::new("127.0.0.1:1337".to_string());
        net2.add_transport( Box::new(udp2.clone()) );

        udp2.seed_address_from_string( "127.0.0.1:12345".to_string() );

        thread::sleep( time::Duration::from_secs(50) );
    });

    t2.join();
}
