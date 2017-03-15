extern crate unbase;
use unbase::subject::*;

#[test]
fn acyclic() {

    let net = unbase::Network::new();
    let simulator = unbase::network::transport::Simulator::new();
    net.add_transport( Box::new(simulator.clone()) );

    let slab_a = unbase::Slab::new(&net);
    let context_a = slab_a.create_context();

    let record1 = Subject::new_blank(&context_a).unwrap();
    let record2 = Subject::new_blank(&context_a).unwrap();
    let record3 = Subject::new_blank(&context_a).unwrap();
    let record4 = Subject::new_blank(&context_a).unwrap();
    let record5 = Subject::new_blank(&context_a).unwrap();
    let record6 = Subject::new_blank(&context_a).unwrap();

    record2.set_relation(0,&record1);
    record3.set_relation(0,&record1);
    record4.set_relation(0,&record1);
    record5.set_relation(0,&record2);
    record6.set_relation(0,&record5);

    //for (subject_id,mrh) in context_a.topo_subject_head_iter(){
    //    println!("Subject {} MRH {:?}", subject_id, mrh );
    //}

}
