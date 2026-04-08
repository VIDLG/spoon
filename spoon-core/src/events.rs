use serde::Serialize;
use tokio::sync::broadcast;

/// Domain event types published by backend workflows.
#[derive(Debug, Clone, Serialize)]
pub enum SpoonEvent {
    Stage(StageEvent),
    Progress(ProgressEvent),
    Notice(NoticeEvent),
    Finished(FinishEvent),
}

/// Sender side of the event bus. Clone-able; send to multiple receivers.
#[derive(Clone)]
pub struct EventSender {
    tx: broadcast::Sender<SpoonEvent>,
}

impl std::fmt::Debug for EventSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventSender")
            .field("receiver_count", &self.tx.receiver_count())
            .finish()
    }
}

/// Receiver side of the event bus. Each clone receives events independently.
pub struct EventReceiver {
    rx: broadcast::Receiver<SpoonEvent>,
}

impl std::fmt::Debug for EventReceiver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventReceiver").finish()
    }
}

impl EventSender {
    /// Publish an event. Returns Ok even if no receivers are listening.
    pub fn send(&self, event: SpoonEvent) {
        // broadcast::send errors when no receivers exist — that's fine.
        let _ = self.tx.send(event);
    }
}

impl EventReceiver {
    /// Receive the next event. Returns Err when all senders are dropped.
    pub async fn recv(&mut self) -> Result<SpoonEvent, broadcast::error::RecvError> {
        self.rx.recv().await
    }

    /// Non-blocking receive. Returns Ok(Some(event)) or Ok(None) if empty.
    pub fn try_recv(&mut self) -> Result<Option<SpoonEvent>, broadcast::error::TryRecvError> {
        match self.rx.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(broadcast::error::TryRecvError::Empty) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

/// Create a new event bus returning (sender, receiver).
/// The sender can be cloned for multiple producers.
/// Call `sender.subscribe()` to get additional receivers.
pub fn event_bus(buffer: usize) -> (EventSender, EventReceiver) {
    let (tx, rx) = broadcast::channel(buffer);
    (
        EventSender { tx },
        EventReceiver { rx },
    )
}

impl EventSender {
    /// Subscribe to events from this sender. Returns a new independent receiver.
    pub fn subscribe(&self) -> EventReceiver {
        EventReceiver {
            rx: self.tx.subscribe(),
        }
    }
}

// ── Event payload types ──

/// Command completion status.
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

#[derive(Debug, Clone, Serialize)]
pub struct FinishEvent {
    pub status: CommandStatus,
    pub message: Option<String>,
    pub code: Option<String>,
}

impl FinishEvent {
    pub fn new(status: CommandStatus, message: Option<String>) -> Self {
        Self {
            status,
            message,
            code: None,
        }
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

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StageEvent {
    pub id: Option<String>,
    pub stage: LifecycleStage,
    pub state: ProgressState,
}

impl StageEvent {
    pub fn new(stage: LifecycleStage, state: ProgressState) -> Self {
        Self {
            id: None,
            stage,
            state,
        }
    }

    pub fn started(stage: LifecycleStage) -> Self {
        Self::new(stage, ProgressState::Running)
    }

    pub fn completed(stage: LifecycleStage) -> Self {
        Self::new(stage, ProgressState::Completed)
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressKind {
    Git,
    Download,
    Cache,
    Extract,
    Work,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressEvent {
    pub id: Option<String>,
    pub kind: ProgressKind,
    pub label: String,
    pub current: Option<u64>,
    pub total: Option<u64>,
    pub unit: ProgressUnit,
}

impl ProgressEvent {
    pub fn new(
        kind: ProgressKind,
        label: impl Into<String>,
        current: Option<u64>,
        total: Option<u64>,
        unit: ProgressUnit,
    ) -> Self {
        Self {
            id: None,
            kind,
            label: label.into(),
            current,
            total,
            unit,
        }
    }

    pub fn bytes(
        kind: ProgressKind,
        label: impl Into<String>,
        current: u64,
        total: Option<u64>,
    ) -> Self {
        Self::new(kind, label, Some(current), total, ProgressUnit::Bytes)
    }

    pub fn items(kind: ProgressKind, label: impl Into<String>, current: u64, total: u64) -> Self {
        Self::new(kind, label, Some(current), Some(total), ProgressUnit::Items)
    }

    pub fn steps(kind: ProgressKind, label: impl Into<String>, current: u64, total: u64) -> Self {
        Self::new(kind, label, Some(current), Some(total), ProgressUnit::Steps)
    }

    pub fn activity(kind: ProgressKind, label: impl Into<String>) -> Self {
        Self::new(kind, label, None, None, ProgressUnit::Unknown)
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NoticeEvent {
    pub level: NoticeLevel,
    pub message: String,
    pub code: Option<String>,
}

impl NoticeEvent {
    pub fn new(level: NoticeLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            code: None,
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(NoticeLevel::Info, message)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(NoticeLevel::Warning, message)
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

/// Lifecycle stages for workflow progression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleStage {
    Planned,
    Detecting,
    Resolving,
    Executing,
    Validating,
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
            Self::Detecting => "detecting",
            Self::Resolving => "resolving",
            Self::Executing => "executing",
            Self::Validating => "validating",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeLevel {
    Info,
    Warning,
}

/// Convenience constants for progress kinds.
pub mod progress_kind {
    use super::ProgressKind;

    pub const GIT: ProgressKind = ProgressKind::Git;
    pub const DOWNLOAD: ProgressKind = ProgressKind::Download;
    pub const CACHE: ProgressKind = ProgressKind::Cache;
    pub const EXTRACT: ProgressKind = ProgressKind::Extract;
}
