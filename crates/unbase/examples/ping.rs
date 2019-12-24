extern crate unbase;
//use std::time;

fn main() {
//    block_on(run())
}
//
//async fn run (){
//    let net = unbase::Network::create_new_system();
//    let udp = unbase::network::transport::TransportUDP::new("127.0.0.1:12345".to_string());
//    net.add_transport( Box::new(udp.clone()) );
//
//    let slab_a = unbase::Slab::new(&net);
//    let context_a = slab_a.create_context();
//    let rec_a1 = Subject::new_kv(&context_a, "animal_sound", "Meow").await.unwrap();
//
//    // ************************************************************************
//    // NOTE: have to use polling for now to detect when the subject has changed
//    // because push notification (though planned) isn't implemented yet :)
//    // ************************************************************************
//
//    let half_sec = time::Duration::from_millis(500);
//    let ten_ms = time::Duration::from_millis(10);
//
//    // spawn thread 1
//    let t = thread::spawn(move || {
//        // use the original copy of the subject, or look it up by sub
//
//        for _ in 1..5 {
//            // Hacky-polling approach for now, push notification coming sooooon!
//            loop {
//                if "Meow".to_string() == rec_a1.get_value("animal_sound").unwrap() {
//                    // set a value when a change is detected
//
//                    info!("[[[ Woof ]]]");
//                    rec_a1.set_value("animal_sound","Woof");
//                    break;
//                }
//                thread::sleep(ten_ms);
//            }
//        }
//    });
//
//
//    t.join().unwrap();
//
//}
