use std::cell::RefCell;
use std::rc::Rc;
use log::{info};
use wasm_bindgen::{JsCast,prelude::*};
use web_sys::{WebGlRenderingContext};

mod state;
mod color;
mod controls;
mod canvas;

pub use self::state::State;
pub use self::color::Color;
pub use self::controls::Controls;
pub use self::canvas::Canvas;

//mod assets;
//pub use self::assets::*;


//use self::render::*;
//use crate::load_texture_img::load_texture_image;

/// Used to instantiate our application
pub struct App {
    //assets: Assets,
    state: RefCell<State>,
    canvas: Canvas,
    controls: Controls
//    renderer: WebRenderer,
}

impl App {
    /// Create a new instance of the Beacon Clock Sim application
    pub fn new() -> Result<Rc<App>, JsValue> {
        let me = Rc::new(App {
            state: RefCell::new(State::new()),
            canvas: Canvas::new()?,
            controls: Controls::new()?
            //assets,
        });

        me.canvas.init_app(me.clone())?;
        me.controls.init_app(me.clone())?;

        Ok(me)
    }
}

impl Rc<App> {
    /// Start our WebGL Water application. `index.html` will call this function in order
    /// to begin rendering.
    pub fn start(&self) -> Result<(), JsValue> {
//        let canvas = &self.canvas;

        info!("beacon-clock-sim WebClient started");

//        load_texture_image(
//            Rc::clone(gl),
//            "/disc.png",
//            TextureUnit::Disc,
//        );

        Ok(())
    }

    pub fn run (&self, run: bool) {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        {
            let mut state = self.state.borrow_mut();
            if state.run == run {
                return;
            }
            state.run = run;
        }

        let app = *self.clone();
        let mut last_time = js_sys::Date::now();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {

            let new_time = js_sys::Date::now(); // Instant::now();
            info!("animation frame");
            let elapsed = last_time - new_time; //new_now.duration_since(last_time).as_millis();
            app.update(elapsed as f32);
            app.render();

            last_time = new_time;

            // Schedule ourself for another requestAnimationFrame callback.
            if app.state.borrow().run {
                request_animation_frame(f.borrow().as_ref().unwrap());
            }
        }) as Box<dyn FnMut()>));

        // Kick things off
        request_animation_frame(g.borrow().as_ref().unwrap());

    }

    /// Update our simulation
    pub fn update(&self, dt: f32) {
        // TODO - change over to logical clock ticks
//        self.app.store.borrow_mut().msg(&Msg::AdvanceClock(dt));
    }

    /// Render the scene. `index.html` will call this once every requestAnimationFrame
    pub fn render(&self) {
//        info!("beacon-clock-sim render");
//        self.renderer.render(&self.gl, &self.app.store.borrow().state, &self.app.assets());
    }
}

pub enum Msg {
    AdvanceClock(f32),
    MouseDown(i32, i32),
    MouseUp,
    MouseMove(i32, i32),
    Zoom(f32),
    BehaviorChange(BehaviorChange),
    Reset()
}


pub enum BehaviorChange{
    Speed(u32),
    Slabs(u32),
    Neighbors(u32),
    Chattyness(f32),
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
