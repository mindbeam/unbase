use futures_util::future::{FutureExt, RemoteHandle};
use futures::Future;

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_with_handle<F: Future> (f: F) -> RemoteHandle<F> {
    let (remote, handle) = f.remote_handle();
    tokio::spawn( remote );

    handle
}

#[cfg(target_arch = "wasm32")]
pub fn spawn_with_handle<F: Future> (f: F)  -> RemoteHandle<F> {
    let (remote, handle) = f.remote_handle();

    wasm_bindgen_futures::spawn_local( remote );

    handle
}