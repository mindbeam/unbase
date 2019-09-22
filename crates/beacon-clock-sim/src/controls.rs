use crate::canvas::APP_DIV_ID;
use crate::App;
//use crate::Msg;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::window;
use web_sys::Element;
use web_sys::HtmlElement;
use web_sys::HtmlInputElement;
use log::{info};
use crate::util::panel::*;

pub fn append_controls(app: Rc<App>) -> Result<(), JsValue> {
    let mut panel = Panel::new(APP_DIV_ID);

    // Run
    panel.add_control(
        Checkbox { start_checked: true, label: "Run" },
        Box::new(move |event: web_sys::Event, element: web_sys::HtmlInputElement| {
            let run : bool = element.checked();
            info!("run change {}", run);
        })
    )?;

    // Speed
    panel.add_control(
        Slider { min: 0.0, max: 1.0, step: 0.01, start: 0.5, label: "Speed" },
        Box::new(move |event: web_sys::Event, element: web_sys::HtmlInputElement| {
            let value : f32 = element.value().parse().unwrap();
            info!("speed change {}", value);
        })
    )?;

    // Slabs
    panel.add_control(
        Slider { min: 0.0, max: 10000.0, step: 1.0, start: 0.5, label: "Slabs" },
        Box::new(move |event: web_sys::Event, element: web_sys::HtmlInputElement| {
            let value : u32 = element.value().parse().unwrap();
            info!("slabs change {}", value);
        })
    )?;

    // 3D
    panel.add_control(
        Checkbox { start_checked: false, label: "3D" },
        Box::new(move |event: web_sys::Event, element: web_sys::HtmlInputElement| {
            let value : bool = element.checked();
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
            let value : u32 = element.value().parse().unwrap();
            info!("Chattyness change {}", value);
        })
    )?;

    // Neighbors
    panel.add_control(
        Slider { min: 0.0, max: 10000.0, step: 1.0, start: 0.5, label: "Neighbors" },
        Box::new(move |event: web_sys::Event, element: web_sys::HtmlInputElement| {
            let value : u32 = element.value().parse().unwrap();
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

//fn create_fresnel_control(app: Rc<App>) -> Result<HtmlElement, JsValue> {
//    let handler = move |event: web_sys::Event| {
//        let input_elem: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
//        let fresnel = input_elem.value().parse().unwrap();
//
////        app.store.borrow_mut().msg(&Msg::SetFresnel(fresnel));
//    };
//    let closure : u32 = Closure::wrap(Box::new(handler) as Box<FnMut(_)>);
//
//    let fresnel_control = Slider {
//        min: 0.0,
//        max: 10.0,
//        step: 0.1,
//        start: 1.5,
//        label: "Fresnel Effect",
//        closure,
//    }
//    .create_element()?;
//
//    Ok(fresnel_control)
//}