use super::{App,Message};
use wasm_bindgen::{JsCast,JsValue};
use web_sys::WebGlRenderingContext as GL;
use web_sys::*;
use wasm_bindgen::prelude::Closure;

use crate::util::*;

pub static APP_DIV_ID: &'static str = "beacon-clock-sim";

pub static CANVAS_WIDTH: i32 = 512;
pub static CANVAS_HEIGHT: i32 = 512;

pub struct Canvas {
    pub gl: WebGlRenderingContext,
    element: HtmlCanvasElement,
}

impl Canvas {
    pub fn new() -> Result<Canvas, JsValue> {
        let element: HtmlCanvasElement = document().create_element("canvas").unwrap().dyn_into()?;

        element.set_width(CANVAS_WIDTH as u32);
        element.set_height(CANVAS_HEIGHT as u32);

        let gl: WebGlRenderingContext = element.get_context("webgl")?.unwrap().dyn_into()?;

        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.enable(GL::DEPTH_TEST);

        Ok(Canvas { gl, element })
    }

    pub fn init_app(&mut self, app: App) -> Result<(), JsValue> {

        self.attach_mouse_down_handler(app.clone())?;
        self.attach_mouse_up_handler(app.clone())?;
        self.attach_mouse_move_handler(app.clone())?;
        self.attach_mouse_wheel_handler(app.clone())?;

        self.attach_touch_start_handler(app.clone())?;
        self.attach_touch_move_handler(app.clone())?;
        self.attach_touch_end_handler(app.clone())?;


        let document = document();

        let app_div: HtmlElement = match document.get_element_by_id(APP_DIV_ID) {
            Some(container) => container.dyn_into()?,
            None => {
                let app_div = document.create_element("div")?;
                app_div.set_id(APP_DIV_ID);
                app_div.dyn_into()?
            }
        };

        app_div.style().set_property("display", "flex")?;
        app_div.append_child(&self.element)?;

        Ok(())
    }
    pub fn width(&self) -> u32 {
        self.element.width()
    }
    pub fn height(&self) -> u32 {
        self.element.height()
    }

    fn attach_mouse_down_handler(&mut self, app: App) -> Result<(), JsValue> {
        let handler = move |event: web_sys::MouseEvent| {
            let x = event.client_x();
            let y = event.client_y();
            app.message(&Message::MouseDown(x, y));
        };

        let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);

        self.element.add_event_listener_with_callback("mousedown", handler.as_ref().unchecked_ref())?;

        handler.forget();

        Ok(())
    }

    fn attach_mouse_up_handler(&mut self, app: App) -> Result<(), JsValue> {
        let handler = move |_event: web_sys::MouseEvent| {
            app.message(&Message::MouseUp);
        };

        let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);

        self.element.add_event_listener_with_callback("mouseup", handler.as_ref().unchecked_ref())?;
        handler.forget();
        Ok(())
    }

    fn attach_mouse_move_handler(&mut self, app: App) -> Result<(), JsValue> {
        let handler = move |event: web_sys::MouseEvent| {
            event.prevent_default();
            let x = event.client_x();
            let y = event.client_y();

            app.message(&Message::MouseMove(x, y));
        };

        let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
        self.element.add_event_listener_with_callback("mousemove", handler.as_ref().unchecked_ref())?;
        handler.forget();

        Ok(())
    }

    fn attach_mouse_wheel_handler(&mut self, app: App) -> Result<(), JsValue> {
        let handler = move |event: web_sys::WheelEvent| {
            event.prevent_default();

            let zoom_amount = event.delta_y() / 50.;

            app.message(&Message::Zoom(zoom_amount as f32));
        };

        let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
        self.element.add_event_listener_with_callback("wheel", handler.as_ref().unchecked_ref())?;
        handler.forget();

        Ok(())
    }

    fn attach_touch_start_handler(&mut self, app: App) -> Result<(), JsValue> {
        let handler = move |event: web_sys::TouchEvent| {
            let touch = event.touches().item(0).expect("First Touch");
            let x = touch.client_x();
            let y = touch.client_y();
            app.message(&Message::MouseDown(x, y));
        };

        let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
        self.element.add_event_listener_with_callback("touchstart", handler.as_ref().unchecked_ref())?;
        handler.forget();

        Ok(())
    }

    fn attach_touch_move_handler(&mut self, app: App) -> Result<(), JsValue> {
        let handler = move |event: web_sys::TouchEvent| {
            event.prevent_default();
            let touch = event.touches().item(0).expect("First Touch");
            let x = touch.client_x();
            let y = touch.client_y();
            app.message(&Message::MouseMove(x, y));
        };

        let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
        self.element.add_event_listener_with_callback("touchmove", handler.as_ref().unchecked_ref())?;
        handler.forget();

        Ok(())
    }

    fn attach_touch_end_handler(&mut self, app: App) -> Result<(), JsValue> {
        let handler = move |_event: web_sys::TouchEvent| {
            app.message(&Message::MouseUp);
        };

        let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);

        self.element.add_event_listener_with_callback("touchend", handler.as_ref().unchecked_ref())?;

        handler.forget();

        Ok(())
    }
}