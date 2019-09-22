
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


use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// This function is automatically invoked after the wasm module is instantiated.
#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {


    init();
    // Here we want to call `requestAnimationFrame` in a loop, but only a fixed
    // number of times. After it's done we want all our resources cleaned up. To
    // achieve this we're using an `Rc`. The `Rc` will eventually store the
    // closure we want to execute on each frame, but to start out it contains
    // `None`.
    //
    // After the `Rc` is made we'll actually create the closure, and the closure
    // will reference one of the `Rc` instances. The other `Rc` reference is
    // used to store the closure, request the first frame, and then is dropped
    // by this function.
    //
    // Inside the closure we've got a persistent `Rc` reference, which we use
    // for all future iterations of the loop
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut i = 0;
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
//        if i > 300 {
//            body().set_text_content(Some("All done!"));
//
//            // Drop our handle to this closure so that it will get cleaned
//            // up once we return.
//            let _ = f.borrow_mut().take();
//            return;
//        }

        // Set the body's text content to how many times this
        // requestAnimationFrame callback has fired.
//        i += 1;
//        let text = format!("requestAnimationFrame has been called {} times.", i);
//        body().set_text_content(Some(&text));
        animate();

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

#[wasm_bindgen(raw_module = "../web/index.ts")]
extern "C" {
    fn init();
    fn animate();
}
