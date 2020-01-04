#![feature(async_closure)]

extern crate unbase;
use unbase::subject::*;
use futures_await_test::async_test;
use tracing::debug;

#[async_test]
async fn basic_record_retrieval() {

    let net = unbase::Network::create_new_system();
    let slab_a = unbase::Slab::new(&net);
    let context_a = slab_a.create_context();

    let record_id;
    {
        let record = Subject::new_kv(&context_a, "animal_type","Cat").await.unwrap();

        debug!("Record {:?}", record );
        record_id = record.id;
    }

    let record_retrieved = context_a.get_subject_by_id(record_id).await;

    assert!(record_retrieved.is_ok(), "Failed to retrieve record")

}

#[async_test]
async fn basic_record_retrieval_simulator() {

    let net = unbase::Network::create_new_system();
    let simulator = unbase::util::simulator::Simulator::new();
    net.add_transport( Box::new(simulator.clone()) );

    let slab_a = unbase::Slab::new(&net);
    let context_a = slab_a.create_context();

    let record_id;
    {
        let record = Subject::new_kv(&context_a, "animal_type","Cat").await.unwrap();

        debug!("Record {:?}", record );
        record_id = record.id;
    }

    let record_retrieved = context_a.get_subject_by_id(record_id).await;

    assert!(record_retrieved.is_ok(), "Failed to retrieve record")

}
