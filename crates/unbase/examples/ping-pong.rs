extern crate unbase;

use unbase::{
    Network,
    Subject,
    subject::SubjectId,
};

use std::time::Duration;
use timer::Delay;
use futures::join;
use unbase::{
    util::simulator::Simulator,
    network::transport::simulator::MemoPayload,
};
use unbase::context::Context;

#[async_std::main]
async fn main () {
    unbase_test_util::init_test_logger();

    let simulator = unbase::util::simulator::Simulator::new();
    let net = unbase::Network::create_new_system();
    net.add_transport(Box::new(simulator.clone()));
    simulator.start();

    let referee_slab = unbase::Slab::new(&net);
    let referee_context = referee_slab.create_context();

    let record : Subject = Subject::new_kv(&referee_context, "the_ball_goes", "PING").await.unwrap();
    let record_id : SubjectId = record.id;

    simulator.quiesce().await;

    let p1 = new_player( net.clone(), simulator.clone(), record_id, "PING", "PONG");
    let p2 = new_player( net.clone(), simulator.clone(), record_id, "PONG", "PING");

    join!{ p1, p2 };

    simulator.quiesce_and_stop().await;

}

async fn new_player(net: Network, sim: Simulator<MemoPayload>, record_id: SubjectId, listen_for: &'static str, then_say: &'static str ) {
    let player_slab = unbase::Slab::new(&net);
    let player_context = player_slab.create_context();

//     TODO - replace this with slab.on_connected().await
    sim.quiesce().await;

    let rec = player_context.get_subject_by_id(record_id).await.expect("Couldn't find the record");

    for _ in 0..5 { // send 5 volleys
        for _ in 1..100 {
            // TODO: update to use push notification (which isn't implemented yet)
            // have to use polling for now to detect when the subject has changed
            Delay::new(Duration::from_millis(50)).await;

            let value = rec.get_value("the_ball_goes").await.unwrap();

            if &value == listen_for  {
                // set a value when a change is detected
                println!("{}", then_say);
                rec.set_value("the_ball_goes", then_say).await;
                break;
            }
        }
    }
}
