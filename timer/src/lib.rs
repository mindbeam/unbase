//#[cfg(not(target_arch = "wasm32"))]
//pub use crate::standard::*;

#[cfg(target_arch = "wasm32")]
pub use crate::wasm::*;

#[cfg(target_arch = "wasm32")]
mod wasm;
