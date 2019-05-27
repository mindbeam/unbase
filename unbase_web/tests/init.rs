#![feature(await_macro, async_await)]

//use futures::compat::Future01CompatExt;
//use futures::compat::Compat;
//use pin_utils::pin_mut;
use futures::future::{FutureExt, TryFutureExt};

use std::time::Duration;
use timer::Timeout;

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use log::*;

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
fn pass(){
    unbase_web::init_logger();
    assert_eq!(1, 1)

}


#[wasm_bindgen_test(async)]
fn pass_after_2s_shim() -> impl futures01::future::Future<Item = (), Error = JsValue> {
    unbase_web::init_logger();

    pass_after_2s().boxed_local().compat()

}

async fn pass_after_2s() -> Result<(),JsValue> {
    info!("immediate log");

    Timeout::new(Duration::from_secs(1)).await;

    info!("log after 1s");


    Timeout::new(Duration::from_secs(1)).await;

    info!("second log after 1s");


    Timeout::new(Duration::from_secs(1)).await;

    info!("third log after 1s");

    Ok(())
}
