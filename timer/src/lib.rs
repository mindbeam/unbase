#![feature(await_macro, async_await)]

#[cfg(not(target_arch = "wasm32"))]
mod standard;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::standard::Delay;

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
pub use crate::wasm::Delay;