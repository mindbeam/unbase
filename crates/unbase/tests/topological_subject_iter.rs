use unbase::{
    Network,
    Slab,
    SubjectHandle,
    util::simulator::Simulator,
};

#[unbase_test_util::async_test]
async fn acyclic() {
    unbase_test_util::init_test_logger();

    let net = Network::create_new_system();
    let simulator = Simulator::new();
    net.add_transport( Box::new(simulator.clone()) );

    let slab_a = Slab::new(&net);
    let context_a = slab_a.create_context();

    let record1     = SubjectHandle::new_blank(&context_a).await.unwrap();
    let mut record2 = SubjectHandle::new_blank(&context_a).await.unwrap();
    let mut record3 = SubjectHandle::new_blank(&context_a).await.unwrap();
    let mut record4 = SubjectHandle::new_blank(&context_a).await.unwrap();
    let mut record5 = SubjectHandle::new_blank(&context_a).await.unwrap();
    let mut record6 = SubjectHandle::new_blank(&context_a).await.unwrap();

    record2.set_relation(0,&record1).await.unwrap();
    record3.set_relation(0,&record1).await.unwrap();
    record4.set_relation(0,&record1).await.unwrap();
    record5.set_relation(0,&record2).await.unwrap();
    record6.set_relation(0,&record5).await.unwrap();

}
