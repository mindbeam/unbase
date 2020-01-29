extern crate unbase;
use std::time::Duration;
use timer::Delay;
use unbase::SubjectHandle;

#[async_std::main]
async fn main() {
    let net = unbase::Network::create_new_system();
    let udp = unbase::network::transport::TransportUDP::new("127.0.0.1:12345".to_string());
    net.add_transport( Box::new(udp.clone()) );

    let slab = unbase::Slab::new(&net);
    let context = slab.create_context();
    let mut record = SubjectHandle::new_kv(&context, "the_ball_goes", "PING").await.unwrap();

    println!("{:?}", record);
    // ************************************************************************
    // NOTE: have to use polling for now to detect when the subject has changed
    // because push notification (though planned) isn't implemented yet :)
    // ************************************************************************

    // use the original copy of the subject, or look it up by sub

    println!("Serving the ball! (PING)");
    println!("Waiting for PONGs");
    for _ in 0..5 {
        // Hacky-polling approach for now, push notification coming sooooon!
        for _ in 0usize..1000 {
            println!(".");
            let value = record.get_value("the_ball_goes").await.unwrap().unwrap();
            if &value == "PONG" {
                // set a value when a change is detected

                println!("[[[ PONG ]]]");
                record.set_value("the_ball_goes", "PONG").await.unwrap();
                break;
            }

            Delay::new(Duration::from_millis(100)).await;
        }
    }

}
