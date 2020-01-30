extern crate unbase;
use unbase::{
    network::transport::TransportUDP,
    Network,
    Slab,
    SubjectHandle,
};
use futures::{
    StreamExt,
};
use std::time::Duration;
use timer::Delay;

#[async_std::main]
async fn main() {
    let net1 = Network::create_new_system();
    let udp1 = TransportUDP::new("127.0.0.1:12001".to_string());
    net1.add_transport(Box::new(udp1));
    let context_a = Slab::new(&net1).create_context();

    // HACK - need to wait until peering of the root index node is established
    // because we are aren't updating the net's root seed, which is what is being sent when peering is established
    // TODO: establish some kind of positive pressure to push out index nodes
    Delay::new(Duration::from_millis(700)).await;

    println!("A - Sending Initial Ping");
    let mut rec_a1 = SubjectHandle::new_kv(&context_a, "action", "Ping").await.unwrap();

    let mut pings = 0;

    while let Some(_) = rec_a1.observe().next().await {
        // HACK - Presently we are relying on the newly issued index leaf for record consistency, which is applied immediately after this event is sent
        Delay::new(Duration::from_millis(10)).await;

        if "Pong" == rec_a1.get_value("action").await.expect("retrieval").expect("has value") {
            println!("A - [ Ping ->       ]");
            rec_a1.set_value("action", "Ping").await.unwrap();
            pings += 1;

            if pings >= 10 {
                break
            }
        }
    }
}
