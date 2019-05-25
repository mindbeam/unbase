use std::time::Duration;

use futures::prelude::*;
use unbase_web::timeout::Timeout;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

//
//#[cfg(target_os = "wasm32")]
//#[test]
//fn basic_record_retrieval_singlethread() {
//
//    let net = unbase::Network::create_new_system();
//    let slab_a = unbase::slab::storage::Memory::new(&net);
//    let context_a = slab_a.create_context();
//
//    let record_id;
//    {
//        let record = SubjectHandle::new_kv(&context_a, "animal_type","Cat").unwrap();
//
//        println!("Record {:?}", record );
//        record_id = record.id;
//    }
//
//    let record_retrieved = context_a.get_subject_by_id(record_id);
//
//    assert!(record_retrieved.is_ok(), "Failed to retrieve record")
//
//}

#[wasm_bindgen_test]
fn pass() {
    assert_eq!(1, 1);
}


#[wasm_bindgen_test(async)]
fn pass_after_2s() -> impl Future<Item = (), Error = JsValue> {
    console_log!("immediate log");
    Timeout::new(Duration::new(1, 0)).and_then(|()| {
        console_log!("log after 1s");
        Timeout::new(Duration::new(1, 0)).map(|()| {
            console_log!("log at end");
        })
    })
}