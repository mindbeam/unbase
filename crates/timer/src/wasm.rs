
use std::time::Duration;
use wasm_bindgen::prelude::*;
use futures::{Future,task::Poll};
use futures::task::{Context,AtomicWaker};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{Ordering,AtomicBool};


pub struct Delay {
    id: u32,
    inner: Arc<Inner>,
    _closure: Closure<dyn FnMut()>,
}

pub struct Inner {
    set: AtomicBool,
    waker: AtomicWaker,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = setTimeout)]
    fn set_timeout(closure: &Closure<dyn FnMut()>, millis: u32) -> u32;

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

        }) as Box<dyn FnMut()>);

        let id = set_timeout(&cb, millis);

        Delay {
            id: id,
            inner: inner,
            _closure: cb,
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        // Register **before** checking `set` to avoid a race condition
        // that would result in lost notifications.
        self.inner.waker.register(cx.waker());

        if self.inner.set.load(Ordering::SeqCst) {
            Poll::Ready(())
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

    use web_sys::console::log_1;

    use super::Delay;
    use std::time::Duration;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_test::*;

    extern crate futures;
    extern crate js_sys;
    extern crate wasm_bindgen_futures;

    #[wasm_bindgen_test]
    async fn three_one_second_delays_future()  {
        log_1(&JsValue::from_str("immediate log"));

        Delay::new(Duration::from_millis(10)).await;

        log_1(&JsValue::from_str("log after 10ms"));

        Delay::new(Duration::from_millis(10)).await;

        log_1(&JsValue::from_str("second log after 10ms"));

        Delay::new(Duration::from_millis(10)).await;

        log_1(&JsValue::from_str("third log after 10ms"));
    }
}