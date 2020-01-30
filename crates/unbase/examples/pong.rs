use unbase::{
    network::transport::TransportUDP,
    Network,
    Slab,
};
use futures::{
    StreamExt
};

use std::time::Duration;
use timer::Delay;

#[async_std::main]
async fn main(){
    let net2 = Network::new();
    net2.hack_set_next_slab_id(200);

    let udp2 = TransportUDP::new("127.0.0.1:12002".to_string());
    net2.add_transport(Box::new(udp2.clone()));

    let context_b = Slab::new(&net2).create_context();

    udp2.seed_address_from_string("127.0.0.1:12001".to_string());

    println!("B - Waiting for root index seed...");
    context_b.root_index().await.unwrap();

    println!("B - Searching for Ping record...");
    let mut rec_b1 = context_b.fetch_kv("action", "Ping", Duration::from_secs(1)).await.unwrap();
    println!("B - Found Ping record.");

    let mut pongs = 0;
    let mut obs = rec_b1.observe();
    while let Some(_) = obs.next().await {
        // HACK - Presently we are relying on the newly issued index leaf for record consistency, which is applied immediately after this event is sent
        Delay::new(Duration::from_millis(10)).await;

        let value = rec_b1.get_value("action").await.expect("it worked").expect("found");
        if "Ping" == value {
            println!("B - [       <- Pong ]");
            rec_b1.set_value("action", "Pong").await.unwrap();
            pongs += 1;

            if pongs >= 10 {
                break
            }
        }
    }
}
