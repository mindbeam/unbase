// WIP

//#[allow(unused_imports)]
////#[cfg(not(target_arch = "wasm32"))]
//
//#[ctor]
//fn init_basic_logger() {
//    env_logger::init();
//    panic!("meow");
//}
//
//#[cfg(target_arch = "wasm32")]
//mod wasm {
//    use wasm_bindgen::prelude::*;
//
//    #[wasm_bindgen(start)]
//    pub fn init_wasm_logger() {
//        use log;
//        log::set_logger(&wasm_bindgen_console_logger::DEFAULT_LOGGER).unwrap();
//        log::set_max_level(log::LevelFilter::Info);
//    }
//}
//
//pub use ::log::*;