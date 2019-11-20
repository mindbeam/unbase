#[cfg(not(target_arch = "wasm32"))]
mod standard;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::standard::Delay;

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
pub use crate::wasm::Delay;

//
//#[cfg(test)]
//mod tests {
//    use crate::Delay;
//    use std::time::Duration;
//
//    #[async_test]
//    async fn three_one_second_delays_future() {
//        println!("immediate log");
//
//        Delay::new(Duration::from_secs(1)).await;
//
//        println!("log after 10ms");
//
//        Delay::new(Duration::from_millis(10)).await;
//
//        println!("second log after 10ms");
//
//        Delay::new(Duration::from_micros(10)).await;
//
//        println!("third log after 10ms");
//
//        Ok(())
//    }
//}