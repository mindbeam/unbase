extern crate unbase;
use unbase::subject::*;
use futures_await_test::async_test;

#[async_test]
async fn acyclic() {
    unbase_test_util::init_test_logger();

    let net = unbase::Network::create_new_system();
    let simulator = unbase::util::simulator::Simulator::new();
    net.add_transport( Box::new(simulator.clone()) );

    let slab_a = unbase::Slab::new(&net);
    let context_a = slab_a.create_context();

    let record1 = Subject::new_blank(&context_a).await.unwrap();
    let record2 = Subject::new_blank(&context_a).await.unwrap();
    let record3 = Subject::new_blank(&context_a).await.unwrap();
    let record4 = Subject::new_blank(&context_a).await.unwrap();
    let record5 = Subject::new_blank(&context_a).await.unwrap();
    let record6 = Subject::new_blank(&context_a).await.unwrap();

    record2.set_relation(0,&record1).await;
    record3.set_relation(0,&record1).await;
    record4.set_relation(0,&record1).await;
    record5.set_relation(0,&record2).await;
    record6.set_relation(0,&record5).await;

}
