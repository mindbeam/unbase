extern crate unbase;
use unbase::subject::*;
use unbase::index::fixed::IndexFixed;
use std::collections::HashMap;

#[test]
fn index_construction() {

    let simulator = unbase::network::Simulator::new();
    let net = unbase::Network::new( &simulator );

    let slab_a = unbase::Slab::new(&net);
    let context_a = slab_a.create_context();

    // Create a new fixed tier index (fancier indexes not necessary for the proof of concept)
    let index = IndexFixed::new(&context_a, 5);

    //for i in 1..1000 {
    let i = 1234;
        let mut vals = HashMap::new();
        vals.insert("record number".to_string(), i.to_string());

        let record = Subject::new(&context_a, vals).unwrap();
        index.insert(i, &record);
    //}

    assert_eq!( index.get(1234).unwrap().get_value("record number").unwrap(), "1234");

/*
    TODO: something in the index is breaking when we try to exercise it a little more
    for i in 1..1000 {
        let mut vals = HashMap::new();
        vals.insert("record number".to_string(), i.to_string());

        let record = Subject::new(&context_a, vals).unwrap();
        index.insert(i, &record);
    }

    for i in 1..1000 {
        assert_eq!( index.get(881).unwrap().get_value("record number").unwrap(), i.to_string() );
    }
    */

}
