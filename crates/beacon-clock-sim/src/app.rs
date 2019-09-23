use std::cell::RefCell;
use std::rc::Rc;
use log::{info};
use wasm_bindgen::{prelude::*};
use std::ops::Deref;

mod state;
mod controls;
mod canvas;

//mod render;

pub use self::state::{State,Message};
pub use self::color::Color;
pub use self::controls::Controls;
pub use self::canvas::Canvas;
use crate::util::*;

//mod assets;
//pub use self::assets::*;


//use self::render::*;
//use crate::load_texture_img::load_texture_image;

/// Used to instantiate our application
#[derive(Clone)]
pub struct App( Rc<AppInner> );

pub struct AppInner {
    //assets: Assets,
    state: RefCell<State>,
    canvas: RefCell<Canvas>,
    controls: RefCell<Controls>,
//    renderer: WebRenderer,
}

impl App {
    /// Create a new instance of the Beacon Clock Sim application
    pub fn new() -> Result<App, JsValue> {
        let inner = AppInner {
            state: RefCell::new(State::new()),
            canvas: RefCell::new(Canvas::new()?),
            controls: RefCell::new(Controls::new()?)
            //assets,
        };

        let app = App( Rc::new(inner) );

        app.canvas.borrow_mut().init_app(app.clone())?;
        app.controls.borrow_mut().init_app(app.clone())?;

        Ok(app)
    }

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

        self.message(&Message::Reset);
        self.run(true);
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

        let app = self.clone();
        let mut last_time = js_sys::Date::now();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            if !app.state.borrow().run {
                return;
            }
            let new_time = js_sys::Date::now(); // Instant::now();
            info!("animation frame");
            let elapsed = last_time - new_time; //new_now.duration_since(last_time).as_millis();
            app.update(elapsed as f32);
            app.render();

            last_time = new_time;

            // Schedule ourself for another requestAnimationFrame callback.
            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));

        // Kick things off
        request_animation_frame(g.borrow().as_ref().unwrap());

    }

    pub fn message (&self, message: &Message ) {
        self.state.borrow_mut().message(message);
    }

    /// Update our simulation
    pub fn update(&self, _dt: f32) {
        // TODO - change over to logical clock ticks
//        self.app.store.borrow_mut().msg(&Msg::AdvanceClock(dt));
    }

    /// Render the scene. `index.html` will call this once every requestAnimationFrame
    pub fn render(&self) {
//        info!("beacon-clock-sim render");
//        self.renderer.render(&self.gl, &self.app.store.borrow().state, &self.app.assets());
    }
}

impl Deref for App {
    type Target = AppInner;
    fn deref(&self) -> &AppInner {
        &*self.0
    }
}
