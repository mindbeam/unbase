extern crate unbase;
use std::{thread, time};
use futures::executor::block_on;

fn main() {
//    block_on(run());
}
//
//async fn run (){
//    let net = unbase::Network::new();
//    net.hack_set_next_slab_id(200);
//
//    let udp = unbase::network::transport::TransportUDP::new("127.0.0.1:1337".to_string());
//    net.add_transport( Box::new(udp.clone()) );
//
//    let slab_b = unbase::Slab::new(&net);
//
//    let half_sec = time::Duration::from_millis(500);
//    let ten_ms = time::Duration::from_millis(10);
//
//    for i in 0..60 {
//        //TODO make it auto retry seeding
//        udp.seed_address_from_string( "127.0.0.1:12345".to_string() );
//
//        if net.get_root_index_seed(&slab_b).is_some() {
//            break
//        }else if i == 59 {
//            println!("Unable to connect to ping node");
//            std::process::exit(0x0100);
//        }
//        println!("Waiting for ping node...");
//        thread::sleep(half_sec);
//    }
//
//    let context_b = slab_b.create_context();
//
//    let mut maybe_rec_b1 = None;
//    for _ in 1..10 {
//
//        // cheater cheater! ( not yet a blocking version of get_subject )
//        thread::sleep( time::Duration::from_millis(500) );
//
//        // Get a new copy of the same subject from context_b (requires communication)
//        // HACK - hardcoding subject_id is baad!
//        if let Ok(rec) = context_b.get_subject_by_id( 9002 ).await {
//            maybe_rec_b1 = Some(rec);
//            break;
//        }
//        println!("Waiting for subject...");
//        thread::sleep(half_sec);
//    }
//
//    let rec_b1 = match maybe_rec_b1 {
//        Some(r) => r,
//        None =>{
//            println!("unable to retrieve subject");
//            std::process::exit(0x0100);
//        }
//    };
//
//    // ************************************************************************
//    // NOTE: have to use polling for now to detect when the subject has changed
//    // because push notification (though planned) isn't implemented yet :)
//    // ************************************************************************
//
//    // spawn thread
//    let t = thread::spawn(move || {
//
//        for _ in 1..5 {
//            // Hacky-polling approach for now, push notification coming sooooon!
//            loop {
//                if "Woof".to_string() == rec_b1.get_value("animal_sound").await.unwrap() {
//                    // set a value when a change is detected
//                    println!("[[[ Meow ]]]");
//                    rec_b1.set_value("animal_sound","Meow");
//                    break;
//                }
//                thread::sleep(ten_ms);
//            }
//        }
//    });
//
//    t.join().unwrap();
//
//}
