#[cfg(not(target_arch = "wasm32"))]
pub use futures_await_test::async_test;

#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen_test::wasm_bindgen_test as async_test;

#[cfg(test)]
mod tests {
    use super::async_test;

    #[async_test]
    async fn it_works() {}
}
