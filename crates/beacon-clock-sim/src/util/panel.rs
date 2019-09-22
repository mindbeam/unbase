use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::window;
use web_sys::{EventTarget, Element, HtmlElement, HtmlInputElement, HtmlButtonElement};

// TODO: inject style sheet and remove the inline CSS

pub struct Panel{
    element: Element,
}

impl Panel {
    pub fn new (parent: &str) -> Self {
        let window = window().unwrap();
        let document = window.document().unwrap();

        let container: HtmlElement = match document.get_element_by_id(parent) {
            Some(container) => container.dyn_into().expect("Html element"),
            None => document.body().expect("Document body"),
        };

        let element = document.create_element("div").unwrap();
        container.append_child(&element).unwrap();

        let element: HtmlElement = element.dyn_into().unwrap();
        element.style().set_property("padding-left", "5px").unwrap();

        let element: Element = element.dyn_into().unwrap();

        Panel{
            element
        }
    }
    pub fn add_control <T: Control, E: 'static > (&mut self, kind: T, mut handler: Box<dyn FnMut(web_sys::Event, E)> ) -> Result<(), JsValue>
    where E: JsCast {
        let handler2 = move |event: web_sys::Event| {
            handler(event.clone(), event.target().unwrap().dyn_into().unwrap());
        };

        let closure = Closure::wrap(Box::new(handler2) as Box<dyn FnMut(_)>);

        let element = kind.create_element(closure)?;

        self.element.append_child(&element)?;

        Ok(())
    }
}

pub trait Control {
    fn create_element( self, closure: Closure<dyn std::ops::FnMut(web_sys::Event)> ) -> Result<HtmlElement, JsValue>;
}

pub struct Slider {
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub start: f32,
    pub label: &'static str,
}

impl Control for Slider {
    fn create_element(self, closure: Closure<dyn std::ops::FnMut(web_sys::Event)> ) -> Result<HtmlElement, JsValue> {
        let window = window().unwrap();
        let document = window.document().unwrap();

        let slider: HtmlInputElement = document.create_element("input")?.dyn_into()?;
        slider.set_type("range");
        slider.set_min(&format!("{}",   self.min));
        slider.set_max(&format!("{}",   self.max));
        slider.set_step(&format!("{}",  self.step));
        slider.set_value(&format!("{}", self.start));

        slider.set_oninput(Some(closure.as_ref().unchecked_ref()));
        closure.forget();

        let label = document.create_element("div")?;
        label.set_inner_html(self.label);

        let container = document.create_element("div")?;
        container.append_child(&label)?;
        container.append_child(&slider)?;

        let container: HtmlElement = container.dyn_into()?;
        container.style().set_property("margin-bottom", "15px")?;

        Ok(container)
    }
}

pub struct Checkbox {
    pub start_checked: bool,
    pub label: &'static str
}

impl Control for Checkbox {
    fn create_element(self, closure: Closure<dyn std::ops::FnMut(web_sys::Event)> ) -> Result<HtmlElement, JsValue> {
        let window = window().unwrap();
        let document = window.document().unwrap();

        let checkbox: HtmlInputElement = document.create_element("input")?.dyn_into()?;
        checkbox.set_type("checkbox");
        checkbox.set_checked(self.start_checked);

        checkbox.set_oninput(Some(closure.as_ref().unchecked_ref()));
        closure.forget();

        let label = document.create_element("label")?;
        label.set_inner_html(self.label);
        label.append_child(&checkbox)?;

        let container = document.create_element("div")?;
        container.append_child(&label)?;

        let container: HtmlElement = container.dyn_into()?;
        container.style().set_property("margin-bottom", "15px")?;
        container.style().set_property("display", "flex")?;
        container.style().set_property("align-items", "center")?;
        container.style().set_property("cursor", "pointer")?;

        Ok(container)
    }
}

pub struct Button {
    pub label: &'static str
}

impl Control for Button {
    fn create_element(self, closure: Closure<dyn std::ops::FnMut(web_sys::Event)> ) -> Result<HtmlElement, JsValue> {
        let window = window().unwrap();
        let document = window.document().unwrap();

        let button: HtmlButtonElement = document.create_element("button")?.dyn_into()?;


        button.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();

        button.set_inner_html(self.label);

        let button : HtmlElement = button.dyn_into()?;
        button.style().set_property("width", "100%")?;

//        let container = document.create_element("div")?;
//        container.append_child(&button)?;
//
//        let container: HtmlElement = container.dyn_into()?;
        button.style().set_property("margin-bottom", "10px")?;
//        container.style().set_property("display", "flex")?;
//        container.style().set_property("align-items", "center")?;
//        container.style().set_property("cursor", "pointer")?;

        Ok(button)
    }
}