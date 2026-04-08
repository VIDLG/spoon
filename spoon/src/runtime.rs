use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

#[derive(Clone, Default)]
pub struct CancelFlag(Arc<AtomicBool>);

impl CancelFlag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

pub fn spawn_with_sender<T, F>(task: F) -> UnboundedReceiver<T>
where
    T: Send + 'static,
    F: FnOnce(UnboundedSender<T>) + Send + 'static,
{
    let (tx, rx) = mpsc::unbounded_channel();
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn_blocking(move || task(tx));
    } else {
        std::thread::spawn(move || task(tx));
    }

    rx
}

pub fn block_on_sync<F: std::future::Future>(future: F) -> F::Output {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        tokio::task::block_in_place(|| handle.block_on(future))
    } else {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build sync runtime")
            .block_on(future)
    }
}

pub fn test_block_on<F: std::future::Future>(future: F) -> F::Output {
    block_on_sync(future)
}
