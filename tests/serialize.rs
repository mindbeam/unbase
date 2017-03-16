extern crate unbase;
use unbase::subject::*;

#[macro_use]
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

#[test]
fn serialize() {

    let net = unbase::Network::new();
    let simulator = unbase::network::transport::Simulator::new();
    net.add_transport( Box::new(simulator.clone()) );

    let slab_a = unbase::Slab::new(&net);
    let context_a = slab_a.create_context();

    let record = Subject::new_kv(&context_a, "animal_type","Cat").unwrap();


    //let limit = bincode::SizeLimit::Bounded(20);
    let memo = record.get_head().to_vec()[0].get_memo(&slab_a).unwrap();

    let encoded = serde_json::to_string(&memo).unwrap();
    //let decoded = serde_json::from_str(&encoded).unwrap();

print!("{}", encoded );
    //let encoded: Vec<u8>        = serialize(&memo).unwrap();
    //let decoded: Option<String> = deserialize(&encoded[..]).unwrap();
//assert_eq!(target, decoded);

}
