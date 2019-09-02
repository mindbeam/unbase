
pub mod util;

//use wasm_bindgen::prelude::*;
//use wasm_bindgen_futures::futures_0_3::*;

use log::{error, info, warn};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    log::set_logger(&wasm_bindgen_console_logger::DEFAULT_LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    info!("unbase_web loaded");
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    info!("Hello, {}!", name)
//    use web_sys::console;
//    console::log_1(&JsValue::from_str(format!("Hello, {}!", name).as_str()));
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}