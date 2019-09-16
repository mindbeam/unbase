//! Unbase beacon clock simulator
//!

#![deny(missing_docs)]
#![feature(custom_attribute)]

extern crate wasm_bindgen;

pub (in crate) use self::app::*;
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

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut last_time = js_sys::Date::now();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {

        let new_time = js_sys::Date::now(); // Instant::now();
        let elapsed = last_time - new_time; //new_now.duration_since(last_time).as_millis();
        webclient.update(elapsed as f32);
        webclient.render();

        last_time = new_time;

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    // Kick things off
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
            "/disc.png",
            TextureUnit::Disc,
        );

        Ok(())
    }

    /// Update our simulation
    pub fn update(&self, dt: f32) {
        // TODO - change over to logical clock ticks
        self.app.store.borrow_mut().msg(&Msg::AdvanceClock(dt));
    }

    /// Render the scene. `index.html` will call this once every requestAnimationFrame
    pub fn render(&mut self) {
        self.renderer
            .render(&self.gl, &self.app.store.borrow().state, &self.app.assets());
    }
}
