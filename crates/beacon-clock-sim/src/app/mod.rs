//use std::cell::RefCell;
//use std::rc::Rc;

//mod store;
mod color;
//pub use self::store::*;
pub use self::color::*;

//mod assets;
//pub use self::assets::*;

/// Used to instantiate our application
pub struct App {
    //assets: Assets,
//    pub store: Rc<RefCell<Store>>,
}

impl App {
    /// Create a new instance of our WebGL Water application
    pub fn new() -> App {
//        let assets = Assets::new();

        App {
            //assets,
//            store: Rc::new(RefCell::new(Store::new())),
        }
    }

//    pub fn assets(&self) -> &Assets {
////        unimplemented!()
//        &self.assets
//    }
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