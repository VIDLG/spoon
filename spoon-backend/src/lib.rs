use serde::Serialize;

pub mod context;
pub mod control_plane;
mod error;
mod event;
mod fsx;
mod gitx;
pub mod layout;
pub mod msvc;
pub mod ports;
mod proxy;
pub mod scoop;
pub mod status;
mod task;

// Test utilities and internal integration tests
#[cfg(test)]
mod tests;

pub use context::BackendContext;
pub use error::{BackendError, Result};
pub use event::{
    BackendEvent, EventReceiver, EventSender, EventSink, FinishEvent, ProgressEvent, ProgressState,
    ProgressUnit,
};
pub use fsx::directory_size;
pub use gitx::{RepoSyncOutcome, clone_repo};
pub use layout::{ManagedMsvcLayout, MsvcLayout, OfficialMsvcLayout, RuntimeLayout, ScoopLayout};
pub use ports::SystemPort;
pub use proxy::{ReqwestClientBuilder, normalize_proxy_url};
pub use scoop::{ScoopIntegrationPort, SupplementalShimSpec};
pub use task::{
    CancellationToken, TaskCancellation, await_task_with_events, check_token_cancel,
    is_token_cancelled, spawn_interrupt_on_cancel,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandStatus {
    Success,
    Failed,
    Cancelled,
    Blocked,
}

impl CommandStatus {
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Success)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Blocked => "blocked",
        }
    }
}
