use std::sync::Once;
static INIT: Once = Once::new();

#[cfg(not(target_arch = "wasm32"))]
pub fn init_basic_logger() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

#[cfg(target_arch = "wasm32")]
pub fn init_basic_logger() {
    INIT.call_once(|| {
        log::set_logger(&wasm_bindgen_console_logger::DEFAULT_LOGGER).unwrap();
        log::set_max_level(log::LevelFilter::Info);
    });
}