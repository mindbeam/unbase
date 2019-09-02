
use std::time::Duration;
use wasm_bindgen::prelude::*;
use futures::{Future,Poll};
use futures::task::{Context,AtomicWaker};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{Ordering,AtomicBool};
use std::io;


pub struct Delay {
    id: u32,
    inner: Arc<Inner>,
    _closure: Closure<FnMut()>,
}

pub struct Inner {
    set: AtomicBool,
    waker: AtomicWaker,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = setTimeout)]
    fn set_timeout(closure: &Closure<FnMut()>, millis: u32) -> u32;

    #[wasm_bindgen(js_name = clearTimeout)]
    fn clear_timeout(id: u32);
}

impl Delay {
    pub fn new(dur: Duration) -> Delay {
        let millis = dur
            .as_secs()
            .checked_mul(1000)
            .unwrap()
            .checked_add(dur.subsec_millis() as u64)
            .unwrap() as u32; // TODO: checked cast

        let inner = Arc::new(Inner {
            waker: AtomicWaker::new(),
            set: AtomicBool::new(false)
        });

        let inner2 = inner.clone();

        let cb = Closure::wrap(Box::new(move || {
            inner2.set.store(true, Ordering::SeqCst);
            inner2.waker.wake();

        }) as Box<FnMut()>);

        let id = set_timeout(&cb, millis);

        Delay {
            id: id,
            inner: inner,
            _closure: cb,
        }
    }
}

impl Future for Delay {
    type Output = io::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        // Register **before** checking `set` to avoid a race condition
        // that would result in lost notifications.
        self.inner.waker.register(cx.waker());

        if self.inner.set.load(Ordering::SeqCst) {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }
}

impl Drop for Delay {
    fn drop(&mut self) {
        clear_timeout(self.id);
    }
}

#[cfg(test)]
mod tests {

    use futures::future::{FutureExt, TryFutureExt};
    use wasm_bindgen_test::*;
    use web_sys::console::log_1;
    use wasm_bindgen::JsValue;

    use super::Delay;
    use std::time::Duration;

//    use wasm_bindgen::prelude::*;
//    use wasm_bindgen_futures::futures_0_3::*;


    #[wasm_bindgen_test(async)]
    fn timeout_wasm() -> impl futures01::future::Future<Item=(), Error=JsValue> {

        three_one_second_delays_future().boxed_local().compat()
    }


    async fn three_one_second_delays_future() -> Result<(), JsValue> {
        log_1(&JsValue::from_str("immediate log"));

        Delay::new(Duration::from_secs(1)).await.map_err(|e| e.to_string() )?;

        log_1(&JsValue::from_str("log after 1s"));

        Delay::new(Duration::from_secs(1)).await.map_err(|e| e.to_string() )?;

        log_1(&JsValue::from_str("second log after 1s"));

        Delay::new(Duration::from_secs(1)).await.map_err(|e| e.to_string() )?;

        log_1(&JsValue::from_str("third log after 1s"));

        Ok(())
    }
}