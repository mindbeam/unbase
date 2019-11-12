use super::canvas::APP_DIV_ID;
use crate::App;
use wasm_bindgen::{JsValue};
use web_sys::{Event,HtmlInputElement,HtmlButtonElement};
use log::{info};

use crate::util::panel::*;
use crate::app::Message;

pub struct Controls{}

impl Controls {
    pub fn new () -> Result<Self,JsValue> {
        Ok(Self{})
    }
    pub fn init_app(&self, app: App) -> Result<(), JsValue> {
        let mut panel = Panel::new(APP_DIV_ID);

        // Run
        {
            let app = app.clone();
            panel.add_control(
                Checkbox { start_checked: true, label: "Run" },
                Box::new(move |_event: Event, element: HtmlInputElement| {
                    let run: bool = element.checked();
                    app.run(run);
                    info!("run change {}", run);
                })
            )?;
        }

        // Speed
        panel.add_control(
            Slider { min: 0.0, max: 1.0, step: 0.01, start: 0.5, label: "Speed" },
            Box::new(move |_event: Event, element: HtmlInputElement| {
                let value: f32 = element.value().parse().unwrap();
                info!("speed change {}", value);
            })
        )?;

        // Slabs
        {
            let app = app.clone();
            panel.add_control(
                Slider { min: 0.0, max: 10000.0, step: 1.0, start: 300.0, label: "Slabs" },
                Box::new(move |_event: Event, element: HtmlInputElement| {
                    let value: u32 = element.value().parse().unwrap();
                    info!("slabs change {}", value);
                    app.message(&Message::Slabs(value));
                })
            )?;
        }

        {
            // 3D
            let app = app.clone();
            panel.add_control(
                Checkbox { start_checked: false, label: "3D" },
                Box::new(move |_event: Event, element: HtmlInputElement| {
                    let value: bool = element.checked();
                    info!("3d change {}", value);
                    app.message(&Message::ThreeDim(value));
                })
            )?;
        }

        // Cull elements +/- Z-axis coordinates to show waves in a 3d system (or camera aligned perhaps?)
//        {
//            let app = app.clone();
//            panel.add_control(
//                Slider { min: 0.0, max: 10000.0, step: 1.0, start: 300.0, label: "Z-axis cull" },
//                Box::new(move |_event: Event, element: HtmlInputElement| {
//                    let value: u32 = element.value().parse().unwrap();
//                    info!("slabs change {}", value);
//                    app.message(&Message::Slabs(value));
//                })
//            )?;
//        }

        // Dropper
        panel.add_control(
            Button { label: "Dropper" },
            Box::new(move |_event: Event, _element: HtmlButtonElement| {
                info!("dropper");
            })
        )?;

        // Chattyness
        panel.add_control(
            Slider { min: 0.0, max: 10000.0, step: 1.0, start: 0.5, label: "Chattyness" },
            Box::new(move |_event: Event, element: HtmlInputElement| {
                let value: u32 = element.value().parse().unwrap();
                info!("Chattyness change {}", value);
            })
        )?;

        // Neighbors
        panel.add_control(
            Slider { min: 0.0, max: 10000.0, step: 1.0, start: 0.5, label: "Neighbors" },
            Box::new(move |_event: Event, element: HtmlInputElement| {
                let value: u32 = element.value().parse().unwrap();
                info!("Neighbors change {}", value);
            })
        )?;

        // RandNeighbor
        panel.add_control(
            Button { label: "Rand Neighbor" },
            Box::new(move |_event: Event, _element: HtmlButtonElement| {
                info!("Rand Neighbor");
            })
        )?;

        // ResetColor
        panel.add_control(
            Button { label: "Reset Color" },
            Box::new(move |_event: Event, _element: HtmlButtonElement| {
                info!("Reset Color");
            })
        )?;

        Ok(())
    }
}