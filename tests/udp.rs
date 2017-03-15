extern crate unbase;

#[test]
fn test_udp() {

    let net = unbase::Network::new();
    let udp = unbase::network::transport::Transport_UDP::new("127.0.0.1:12345".to_string());
    net.add_transport( Box::new(udp.clone()) );

    use std::{thread, time};
    thread::sleep( time::Duration::from_secs(50) );
}
