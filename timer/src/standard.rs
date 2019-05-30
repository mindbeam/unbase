pub use futures_timer::Delay;

#[cfg(test)]
mod tests {
    use super::Delay;
    use std::io::Error;
    use std::time::Duration;

    #[test]
    fn timeout_std() {

        futures::executor::block_on(three_one_second_delays_future()).unwrap();
    }

    async fn three_one_second_delays_future() -> Result<(), Error> {
        println!("immediate log");

        Delay::new(Duration::from_secs(1)).await?;

        println!("log after 1s");

        Delay::new(Duration::from_secs(1)).await?;

        println!("second log after 1s");

        Delay::new(Duration::from_secs(1)).await?;

        println!("third log after 1s");

        Ok(())
    }
}