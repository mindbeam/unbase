pub use futures_timer::Delay;

#[cfg(test)]
mod tests {
    use super::Delay;
    use std::time::Duration;

    use futures_await_test::async_test;

    #[async_test]
    async fn three_one_second_delays_future() {
        println!("immediate log");

        Delay::new(Duration::from_secs(1)).await;

        println!("log after 10ms");

        Delay::new(Duration::from_millis(10)).await;

        println!("second log after 10ms");

        Delay::new(Duration::from_micros(10)).await;

        println!("third log after 10ms");

        ()
    }
}