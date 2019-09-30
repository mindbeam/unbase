pub mod panel;
pub mod color;
pub mod position;
pub mod texture;

pub use self::color::Color;
pub use self::position::Position;

use wasm_bindgen::{prelude::*, JsCast};

pub fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

pub fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

//pub fn body() -> web_sys::HtmlElement {
//    document().body().expect("document should have a body")
//}