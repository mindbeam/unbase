use unbase::{
    network::transport::TransportUDP,
    Network,
    Slab,
};
use std::time::Duration;
use timer::Delay;

#[async_std::main]
async fn main(){
    let net = Network::new();
    net.hack_set_next_slab_id(200);

    let udp = TransportUDP::new("127.0.0.1:1337".to_string());
    net.add_transport( Box::new(udp.clone()) );

    let slab = Slab::new(&net);

    for i in 0..60 {
        //TODO make it auto retry seeding
        udp.seed_address_from_string( "127.0.0.1:12345".to_string() );

        if net.get_root_index_seed(&slab).is_some() {
            break
        }else if i == 59 {
            println!("Unable to connect to ping node");
            std::process::exit(0x0100);
        }
        println!("Waiting for ping node...");
        Delay::new(Duration::from_millis(500)).await;
    }

    let context = slab.create_context();

    let mut maybe_record = None;
    for _ in 1..10 {

        // cheater cheater! ( not yet a blocking version of get_subject )
        Delay::new(Duration::from_millis(500)).await;

        // Get a new copy of the same subject from context_b (requires communication)
        // HACK - hardcoding subject_id is baad!
        if let Ok(rec) = context.get_subject_by_id( 9002 ).await {
            maybe_record = Some(rec);
            break;
        }
        println!("Waiting for subject...");
        Delay::new(Duration::from_millis(500)).await;
    }

    let mut record = match maybe_record {
        Some(r) => r,
        None =>{
            println!("unable to retrieve subject");
            std::process::exit(0x0100);
        }
    };

    // ************************************************************************
    // NOTE: have to use polling for now to detect when the subject has changed
    // because push notification (though planned) isn't implemented yet :)
    // ************************************************************************

    for _ in 0..5 {
        // Hacky-polling approach for now, push notification coming sooooon!
        for _ in 0..1000 {

            let value = record.get_value("the_ball_goes").await.unwrap();
            if &value == "PING" {
                // set a value when a change is detected
                println!("[[[ PONG ]]]");
                record.set_value("the_ball_goes","PONG").await;
                break;
            }
            Delay::new(Duration::from_millis(100)).await;
        }
    }
}
