extern crate unbase_util;

#[cfg(not(target_arch = "wasm32"))]
mod standard;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::standard::Delay;

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
pub use crate::wasm::Delay;

#[cfg(test)]
mod tests {
    use crate::Delay;
    use std::time::Duration;

//    use unbase_util::test::async_test;

    #[unbase_util::test::async_test]
    async fn three_one_second_delays_future() {

        let dur = Duration::from_millis(10);
        Delay::new(dur).await;

        Delay::new(dur).await;

        Delay::new(dur).await;

    }
}
