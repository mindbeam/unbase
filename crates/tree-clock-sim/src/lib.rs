//! An example of how to render water using WebGL + Rust + WebAssembly.
//!
//! We'll try to heavily over comment the code so that it's more accessible to those that
//! are less familiar with the techniques that are used.
//!
//! In a real application you'd split things up into different modules and files,
//! but I tend to prefer tutorials that are all in one file that you can scroll up and down in
//! and soak up what you see vs. needing to hop around different files.
//!
//! If you have any questions or comments feel free to open an issue on GitHub!
//!
//! https://github.com/chinedufn/webgl-water-tutorial
//!
//! Heavily inspired by this @thinmatrix tutorial:
//!   - https://www.youtube.com/watch?v=HusvGeEDU_U&list=PLRIWtICgwaX23jiqVByUs0bqhnalNTNZh

#![deny(missing_docs)]
#![feature(custom_attribute)]

extern crate wasm_bindgen;
pub(in crate) use self::app::*;
use self::canvas::*;
use self::controls::*;
use self::render::*;
use crate::load_texture_img::load_texture_image;
use console_error_panic_hook;
use wasm_bindgen::{JsCast,prelude::*};
use std::cell::RefCell;
use std::rc::Rc;

use web_sys::*;
//use std::time::{Duration, Instant};

mod app;
mod canvas;
mod controls;
mod load_texture_img;
mod render;
mod shader;

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn body() -> web_sys::HtmlElement {
    document().body().expect("document should have a body")
}

/// This function is automatically invoked after the wasm module is instantiated.
#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {

    let mut webclient = WebClient::new();

    webclient.start();

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

    let mut last_time = js_sys::Date::now();//Instant::now();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {

        let new_time = js_sys::Date::now(); // Instant::now();
        let elapsed = last_time - new_time; //new_now.duration_since(last_time).as_millis();
        webclient.update(elapsed as f32);
        webclient.render();

        last_time = new_time;

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())

}


/// Used to run the application from the web
pub struct WebClient {
    app: Rc<App>,
    gl: Rc<WebGlRenderingContext>,
    renderer: WebRenderer,
}

impl WebClient {
    /// Create a new web client
    pub fn new() -> WebClient {
        console_error_panic_hook::set_once();

        let app = Rc::new(App::new());

        let gl = Rc::new(create_webgl_context(Rc::clone(&app)).unwrap());
        append_controls(Rc::clone(&app)).expect("Append controls");

        let renderer = WebRenderer::new(&gl);

        WebClient { app, gl, renderer }
    }

    /// Start our WebGL Water application. `index.html` will call this function in order
    /// to begin rendering.
    pub fn start(&self) -> Result<(), JsValue> {
        let gl = &self.gl;

        load_texture_image(
            Rc::clone(gl),
            "/dudvmap.png",
            TextureUnit::Dudv,
        );
        load_texture_image(
            Rc::clone(gl),
            "/normalmap.png",
            TextureUnit::NormalMap,
        );
        load_texture_image(
            Rc::clone(gl),
            "/stone-texture.png",
            TextureUnit::Stone,
        );

        Ok(())
    }

    /// Update our simulation
    pub fn update(&self, dt: f32) {
        self.app.store.borrow_mut().msg(&Msg::AdvanceClock(dt));
    }

    /// Render the scene. `index.html` will call this once every requestAnimationFrame
    pub fn render(&mut self) {
        self.renderer
            .render(&self.gl, &self.app.store.borrow().state, &self.app.assets());
    }
}
