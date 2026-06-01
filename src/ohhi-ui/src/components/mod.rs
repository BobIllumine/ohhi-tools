pub mod board;
pub mod analysis;
pub mod play;
pub mod patterns;
pub mod practice;
pub mod stats;

/// Async sleep that works on both desktop (tokio) and web (wasm). Used to drive
/// the millisecond game/drill timers; without a wasm path the timer froze on the
/// deployed web build.
pub(crate) async fn sleep_ms(ms: u32) {
    #[cfg(not(target_arch = "wasm32"))]
    tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::TimeoutFuture::new(ms).await;
}
