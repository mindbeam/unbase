
use std::time::Duration;
use wasm_bindgen::prelude::*;
use futures::{Future,Poll};
use futures::task::{Context,AtomicWaker};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{Ordering,AtomicBool};

pub struct Timeout {
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

impl Timeout {
    pub fn new(dur: Duration) -> Timeout {
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

        Timeout {
            id: id,
            inner: inner,
            _closure: cb,
        }
    }
}

impl Future for Timeout {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
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

impl Drop for Timeout {
    fn drop(&mut self) {
        clear_timeout(self.id);
    }
}