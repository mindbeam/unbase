
// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::futures_0_3::{JsFuture, future_to_promise, spawn_local};
use web_sys::console;
use std::time::Duration;

use timer::Delay;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen]
pub fn hello_worlx() -> js_sys::Promise {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
        console_error_panic_hook::set_once();

    future_to_promise(hello_worly())
}

async fn hello_worly () -> Result<JsValue,JsValue>{
        // Your code goes here!
        console::log_1(&JsValue::from_str("Hello world!"));

        console::log_1(&JsValue::from_str("Sleeping 1 second"));
        Delay::new(Duration::from_secs(1)).await.map_err(|e| e.to_string())?;

        console::log_1(&JsValue::from_str("Sleeping 1 second"));
        Delay::new(Duration::from_secs(1)).await.map_err(|e| e.to_string())?;

        console::log_1(&JsValue::from_str("Done!"));

        Ok(1.into())
}

pub mod slab;