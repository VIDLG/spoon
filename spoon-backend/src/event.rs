use serde::Serialize;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::CommandStatus;

pub mod progress_kind {
    pub const GIT: &str = "git";
    pub const DOWNLOAD: &str = "download";
    pub const CACHE: &str = "cache";
    pub const EXTRACT: &str = "extract";
    pub const LIFECYCLE: &str = "lifecycle";
}

#[derive(Debug, Clone, Serialize)]
pub enum BackendEvent {
    Progress(ProgressEvent),
    Finished(FinishEvent),
}

pub type EventSender = UnboundedSender<BackendEvent>;
pub type EventReceiver = UnboundedReceiver<BackendEvent>;

pub struct EventSink<'a> {
    emit: Option<&'a mut dyn FnMut(BackendEvent)>,
}

impl<'a> EventSink<'a> {
    pub fn new(emit: Option<&'a mut dyn FnMut(BackendEvent)>) -> Self {
        Self { emit }
    }

    pub fn is_enabled(&self) -> bool {
        self.emit.is_some()
    }

    pub fn send(&mut self, event: BackendEvent) {
        if let Some(emit) = self.emit.as_deref_mut() {
            emit(event);
        }
    }

    pub fn flush(&mut self, receiver: &mut Option<EventReceiver>) {
        let Some(receiver) = receiver.as_mut() else {
            return;
        };
        while let Ok(event) = receiver.try_recv() {
            self.send(event);
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FinishEvent {
    pub status: CommandStatus,
    pub message: Option<String>,
}

impl FinishEvent {
    pub fn new(status: CommandStatus, message: Option<String>) -> Self {
        Self { status, message }
    }

    pub fn success(message: Option<String>) -> Self {
        Self::new(CommandStatus::Success, message)
    }

    pub fn cancelled(message: impl Into<String>) -> Self {
        Self::new(CommandStatus::Cancelled, Some(message.into()))
    }

    pub fn failed(message: impl Into<String>) -> Self {
        Self::new(CommandStatus::Failed, Some(message.into()))
    }

    pub fn blocked(message: impl Into<String>) -> Self {
        Self::new(CommandStatus::Blocked, Some(message.into()))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressEvent {
    pub id: Option<String>,
    pub kind: &'static str,
    pub stage: Option<LifecycleStage>,
    pub state: ProgressState,
    pub label: String,
    pub current: Option<u64>,
    pub total: Option<u64>,
    pub unit: ProgressUnit,
}

impl ProgressEvent {
    pub fn new(
        kind: &'static str,
        label: impl Into<String>,
        current: Option<u64>,
        total: Option<u64>,
        unit: ProgressUnit,
    ) -> Self {
        Self {
            id: None,
            kind,
            stage: None,
            state: ProgressState::Running,
            label: label.into(),
            current,
            total,
            unit,
        }
    }

    pub fn bytes(
        kind: &'static str,
        label: impl Into<String>,
        current: u64,
        total: Option<u64>,
    ) -> Self {
        Self::new(kind, label, Some(current), total, ProgressUnit::Bytes)
    }

    pub fn items(kind: &'static str, label: impl Into<String>, current: u64, total: u64) -> Self {
        Self::new(kind, label, Some(current), Some(total), ProgressUnit::Items)
    }

    pub fn steps(kind: &'static str, label: impl Into<String>, current: u64, total: u64) -> Self {
        Self::new(kind, label, Some(current), Some(total), ProgressUnit::Steps)
    }

    pub fn activity(kind: &'static str, label: impl Into<String>) -> Self {
        Self::new(kind, label, None, None, ProgressUnit::Unknown)
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_stage(mut self, stage: LifecycleStage) -> Self {
        self.stage = Some(stage);
        self
    }

    pub fn with_state(mut self, state: ProgressState) -> Self {
        self.state = state;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleStage {
    Planned,
    Acquiring,
    Materializing,
    PreparingHooks,
    PersistRestoring,
    SurfaceApplying,
    PostInstallHooks,
    Integrating,
    StateCommitting,
    PreUninstallHooks,
    Uninstalling,
    PersistSyncing,
    SurfaceRemoving,
    StateRemoving,
    PostUninstallHooks,
    Completed,
}

impl LifecycleStage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Acquiring => "acquiring",
            Self::Materializing => "materializing",
            Self::PreparingHooks => "preparing_hooks",
            Self::PersistRestoring => "persist_restoring",
            Self::SurfaceApplying => "surface_applying",
            Self::PostInstallHooks => "post_install_hooks",
            Self::Integrating => "integrating",
            Self::StateCommitting => "state_committing",
            Self::PreUninstallHooks => "pre_uninstall_hooks",
            Self::Uninstalling => "uninstalling",
            Self::PersistSyncing => "persist_syncing",
            Self::SurfaceRemoving => "surface_removing",
            Self::StateRemoving => "state_removing",
            Self::PostUninstallHooks => "post_uninstall_hooks",
            Self::Completed => "completed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ProgressUnit {
    Bytes,
    Items,
    Steps,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ProgressState {
    Running,
    Completed,
}
