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
    use log::info;

    #[unbase_test_util::async_test]
    async fn three_one_second_delays_future() {
        unbase_test_util::init_test_logger();

        info!("immediate log");
        let dur = Duration::from_millis(10);
        Delay::new(dur).await;

        info!("log after 10ms");

        Delay::new(dur).await;

        info!("log after 10ms");

        Delay::new(dur).await;

        info!("done");
    }
}
