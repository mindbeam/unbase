
use wasm_bindgen::prelude::*;
use std::sync::Arc;

#[wasm_bindgen]
pub struct XYZPoint{
    pub x: i64,
    pub y: i64,
    pub z: i64
}

impl XYZPoint{
    pub fn random () -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        XYZPoint{ x: rng.gen(), y: rng.gen(), z: rng.gen() }
    }
}

#[wasm_bindgen]
pub struct SimSlab {
    point: XYZPoint,
//    pub slab: unbase_web::Slab,
//    pub color: Color,
//    neighbors: Array<[number,Slab]>,
//    clockstate: MemoHead,
}

#[wasm_bindgen]
impl SimSlab {
    pub fn new() -> SimSlab {
        SimSlab { point: XYZPoint::random() }
    }
//
//    pub fn get(&self) -> i32 {
//        self.internal
//    }
//
//    pub fn set(&mut self, val: i32) {
//        self.internal = val;
//    }
}