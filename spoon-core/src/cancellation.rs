use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{CoreError, Result};
pub use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct TaskCancellation {
    token: Option<CancellationToken>,
    interrupt: Arc<AtomicBool>,
}

impl TaskCancellation {
    pub fn new(token: Option<CancellationToken>) -> Self {
        Self {
            token,
            interrupt: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn token(&self) -> Option<&CancellationToken> {
        self.token.as_ref()
    }

    pub fn interrupt_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.interrupt)
    }

    pub fn is_interrupted(&self) -> bool {
        self.interrupt.load(Ordering::Relaxed)
    }

    pub fn check(&self) -> Result<()> {
        check_token_cancel(self.token())
    }

    pub fn spawn_interrupt_task(&self) -> Option<tokio::task::JoinHandle<()>> {
        spawn_interrupt_on_cancel(self.token.clone(), Arc::clone(&self.interrupt))
    }
}

pub fn is_token_cancelled(cancel: Option<&CancellationToken>) -> bool {
    cancel.is_some_and(CancellationToken::is_cancelled)
}

pub fn check_token_cancel(cancel: Option<&CancellationToken>) -> Result<()> {
    if is_token_cancelled(cancel) {
        return Err(CoreError::Cancelled);
    }
    Ok(())
}

pub fn spawn_interrupt_on_cancel(
    cancel: Option<CancellationToken>,
    should_interrupt: Arc<AtomicBool>,
) -> Option<tokio::task::JoinHandle<()>> {
    cancel.map(move |cancel| {
        tokio::spawn(async move {
            cancel.cancelled().await;
            should_interrupt.store(true, Ordering::Relaxed);
        })
    })
}
