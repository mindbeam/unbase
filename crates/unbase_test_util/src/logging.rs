#[cfg(not(target_arch = "wasm32"))]
pub fn init_test_logger() {
    env_logger::init();
}

#[cfg(target_arch = "wasm32")]
pub fn init_test_logger() {
    log::set_logger(&wasm_bindgen_console_logger::DEFAULT_LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Info);
}