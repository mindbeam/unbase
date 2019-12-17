use futures_util::future::{FutureExt, RemoteHandle};
use futures::Future;

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_with_handle<F> (f: F) -> RemoteHandle<F::Output>
    where F: Future + 'static + Send,
    <F as Future>::Output: std::marker::Send {
    let (remote, handle) = f.remote_handle();

    tokio::spawn( remote );

    handle
}

#[cfg(target_arch = "wasm32")]
pub fn spawn_with_handle<F: Future + 'static + Send> (f: F)  -> RemoteHandle<()> {
    let (remote, handle) = f.remote_handle();

    wasm_bindgen_futures::spawn_local( remote );

    handle
}