//! Unbase beacon clock simulator
//!

#![deny(missing_docs)]
#![feature(custom_attribute)]

extern crate wasm_bindgen;

use log::{info};

pub (in crate) use self::app::*;

use console_error_panic_hook;
use wasm_bindgen::{JsCast,prelude::*};
use std::cell::RefCell;
use std::rc::Rc;

use web_sys::*;
//use std::time::{Duration, Instant};

mod util;
mod app;
//mod load_texture_img;
//mod render;
//mod shader;

/// This function is automatically invoked after the wasm module is instantiated.
#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    log::set_logger(&wasm_bindgen_console_logger::DEFAULT_LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    info!("beacon-clock-sim loaded");

    let mut app = App::new()?;

    app.start()?;

    info!("beacon-clock-sim started");

    Ok(())
}