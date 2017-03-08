extern crate unbase;
use unbase::subject::*;

#[test]
fn basic_record_retrieval() {

    let simulator = unbase::network::Simulator::new();
    let net = unbase::Network::new( &simulator );

    let slab_a = unbase::Slab::new(&net);
    let context_a = slab_a.create_context();

    let record_id;
    {
        let record = Subject::new_kv(&context_a, "animal_type","Cat").unwrap();

        println!("Record {:?}", record );
        record_id = record.id;
    }

    let record_retrieved = context_a.get_subject_by_id(record_id);

    assert!(record_retrieved.is_ok(), "Failed to retrieve record")

}
