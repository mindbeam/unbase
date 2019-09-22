use super::canvas::APP_DIV_ID;
use crate::App;
//use crate::Msg;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{window, Element,HtmlElement,HtmlInputElement};
use log::{info};

use crate::util::panel::*;

pub struct Controls{}

impl Controls {
    pub fn new () -> Result<Self,JsValue> {
        Ok(Self{})
    }
    pub fn init_app(&self, app: Rc<App>) -> Result<(), JsValue> {
        let mut panel = Panel::new(APP_DIV_ID);

        // Run
        panel.add_control(
            Checkbox { start_checked: true, label: "Run" },
            Box::new(move |event: web_sys::Event, element: web_sys::HtmlInputElement| {
                let run: bool = element.checked();
                app.run(run);
                info!("run change {}", run);
            })
        )?;

        // Speed
        panel.add_control(
            Slider { min: 0.0, max: 1.0, step: 0.01, start: 0.5, label: "Speed" },
            Box::new(move |event: web_sys::Event, element: web_sys::HtmlInputElement| {
                let value: f32 = element.value().parse().unwrap();
                info!("speed change {}", value);
            })
        )?;

        // Slabs
        panel.add_control(
            Slider { min: 0.0, max: 10000.0, step: 1.0, start: 0.5, label: "Slabs" },
            Box::new(move |event: web_sys::Event, element: web_sys::HtmlInputElement| {
                let value: u32 = element.value().parse().unwrap();
                info!("slabs change {}", value);
            })
        )?;

        // 3D
        panel.add_control(
            Checkbox { start_checked: false, label: "3D" },
            Box::new(move |event: web_sys::Event, element: web_sys::HtmlInputElement| {
                let value: bool = element.checked();
                info!("3d change {}", value);
            })
        )?;

        // Dropper
        panel.add_control(
            Button { label: "Dropper" },
            Box::new(move |event: web_sys::Event, element: web_sys::HtmlButtonElement| {
                info!("dropper");
            })
        )?;

        // Chattyness
        panel.add_control(
            Slider { min: 0.0, max: 10000.0, step: 1.0, start: 0.5, label: "Chattyness" },
            Box::new(move |event: web_sys::Event, element: web_sys::HtmlInputElement| {
                let value: u32 = element.value().parse().unwrap();
                info!("Chattyness change {}", value);
            })
        )?;

        // Neighbors
        panel.add_control(
            Slider { min: 0.0, max: 10000.0, step: 1.0, start: 0.5, label: "Neighbors" },
            Box::new(move |event: web_sys::Event, element: web_sys::HtmlInputElement| {
                let value: u32 = element.value().parse().unwrap();
                info!("Neighbors change {}", value);
            })
        )?;

        // RandNeighbor
        panel.add_control(
            Button { label: "Rand Neighbor" },
            Box::new(move |event: web_sys::Event, element: web_sys::HtmlButtonElement| {
                info!("Rand Neighbor");
            })
        )?;

        // ResetColor
        panel.add_control(
            Button { label: "Reset Color" },
            Box::new(move |event: web_sys::Event, element: web_sys::HtmlButtonElement| {
                info!("Reset Color");
            })
        )?;

        Ok(())
    }
}