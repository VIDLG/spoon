use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    CoreError, SpoonEvent, CancellationToken, EventSender, ProgressEvent,
    ProgressUnit, Result, TaskCancellation, progress_kind,
    normalize_proxy_url,
};
use gix::progress::{Count, DynNestedProgressToNestedProgress, NestedProgress, Progress};
use gix::progress::{Id as ProgressId, Step as ProgressStep, StepShared, UNKNOWN};
use gix::progress::{MessageLevel, Unit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoSyncOutcome {
    /// HEAD commit hash after sync.
    pub head_commit: Option<String>,
    /// Branch name of HEAD (detached HEAD => None).
    pub head_branch: Option<String>,
}

/// Internal state for GixProgress tracking.
#[derive(Debug)]
struct GixProgressState {
    name: String,
    id: ProgressId,
    max: Option<ProgressStep>,
    unit: ProgressUnit,
    last_emitted_step: Option<ProgressStep>,
}

impl GixProgressState {
    fn label(&self) -> String {
        if !self.name.trim().is_empty() {
            return self.name.clone();
        }
        if self.id != UNKNOWN {
            let rendered = String::from_utf8_lossy(&self.id).replace('\0', "");
            if !rendered.trim().is_empty() {
                return rendered.to_string();
            }
        }
        "git operation".to_string()
    }
}

/// Progress adapter for gix operations that emits backend events.
#[derive(Debug, Clone)]
struct GixProgress {
    events: Option<EventSender>,
    state: Arc<std::sync::Mutex<GixProgressState>>,
    step: StepShared,
}

impl GixProgress {
    fn new_root(events: Option<EventSender>, name: impl Into<String>) -> Self {
        Self::new(events, name.into(), UNKNOWN)
    }

    fn new(events: Option<EventSender>, name: String, id: ProgressId) -> Self {
        Self {
            events,
            state: Arc::new(std::sync::Mutex::new(GixProgressState {
                name,
                id,
                max: None,
                unit: ProgressUnit::Unknown,
                last_emitted_step: None,
            })),
            step: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn get_step(&self) -> ProgressStep {
        self.step.load(Ordering::Relaxed)
    }

    fn set_step(&self, value: ProgressStep) {
        self.step.store(value, Ordering::Relaxed);
        self.emit_progress(false);
    }

    fn inc_step(&self, delta: ProgressStep) {
        self.step.fetch_add(delta, Ordering::Relaxed);
        self.emit_progress(false);
    }

    fn emit_progress(&self, force: bool) {
        let Some(events) = &self.events else {
            return;
        };
        let mut state = self.state.lock().expect("git progress state poisoned");
        let step = self.get_step();
        if !force && state.last_emitted_step == Some(step) {
            return;
        }
        state.last_emitted_step = Some(step);
        let progress = ProgressEvent::new(
            progress_kind::GIT,
            state.label(),
            Some(step as u64),
            state.max.map(|v| v as u64),
            state.unit,
        );
        let progress = if state.id != UNKNOWN {
            let rendered = String::from_utf8_lossy(&state.id).replace('\0', "");
            if rendered.trim().is_empty() {
                progress
            } else {
                progress.with_id(rendered)
            }
        } else {
            progress
        };
        events.send(SpoonEvent::Progress(progress));
    }
}

impl Progress for GixProgress {
    fn init(&mut self, max: Option<ProgressStep>, _unit: Option<Unit>) {
        let mut state = self.state.lock().expect("git progress state poisoned");
        state.max = max;
        let current = state.unit;
        state.unit = infer_progress_unit(&state.name, state.id, current);
    }

    fn max(&self) -> Option<ProgressStep> {
        self.state.lock().ok()?.max
    }

    fn set_max(&mut self, max: Option<ProgressStep>) -> Option<ProgressStep> {
        let mut state = self.state.lock().expect("git progress state poisoned");
        let previous = state.max;
        state.max = max;
        previous
    }

    fn set_name(&mut self, name: String) {
        let mut state = self.state.lock().expect("git progress state poisoned");
        state.name = name;
        let current = state.unit;
        state.unit = infer_progress_unit(&state.name, state.id, current);
    }

    fn name(&self) -> Option<String> {
        Some(self.state.lock().ok()?.name.clone())
    }

    fn id(&self) -> ProgressId {
        self.state.lock().map(|state| state.id).unwrap_or(UNKNOWN)
    }

    fn message(&self, level: MessageLevel, message: String) {
        let label = self.state.lock().ok().and_then(|s| {
            let label = s.label();
            if label.is_empty() { None } else { Some(label) }
        });
        match level {
            MessageLevel::Info | MessageLevel::Success => {
                if let Some(label) = label {
                    tracing::info!(progress_label = %label, "{message}");
                } else {
                    tracing::info!("{message}");
                }
            }
            MessageLevel::Failure => {
                if let Some(label) = label {
                    tracing::warn!(progress_label = %label, "{message}");
                } else {
                    tracing::warn!("{message}");
                }
            }
        }
    }
}

impl NestedProgress for GixProgress {
    type SubProgress = GixProgress;

    fn add_child(&mut self, name: impl Into<String>) -> Self::SubProgress {
        let name = name.into();
        let parent = self.name().unwrap_or_default();
        let label = if parent.trim().is_empty() {
            name
        } else {
            format!("{parent} / {name}")
        };
        GixProgress::new(self.events.clone(), label, UNKNOWN)
    }

    fn add_child_with_id(&mut self, name: impl Into<String>, id: ProgressId) -> Self::SubProgress {
        let name = name.into();
        let parent = self.name().unwrap_or_default();
        let label = if parent.trim().is_empty() {
            name
        } else {
            format!("{parent} / {name}")
        };
        GixProgress::new(self.events.clone(), label, id)
    }
}

impl Count for GixProgress {
    fn set(&self, step: ProgressStep) {
        self.set_step(step);
    }

    fn step(&self) -> ProgressStep {
        self.get_step()
    }

    fn inc_by(&self, step: ProgressStep) {
        self.inc_step(step);
    }

    fn counter(&self) -> StepShared {
        Arc::clone(&self.step)
    }
}

fn infer_progress_unit(name: &str, id: ProgressId, current: ProgressUnit) -> ProgressUnit {
    if current != ProgressUnit::Unknown {
        return current;
    }
    let name_lower = name.to_ascii_lowercase();
    if name_lower.contains("byte") || id == *b"CLCB" {
        ProgressUnit::Bytes
    } else if name_lower.contains("file")
        || name_lower.contains("object")
        || name_lower.contains("ref")
        || id == *b"CLCF"
    {
        ProgressUnit::Items
    } else {
        ProgressUnit::Unknown
    }
}

fn clone_repo_blocking(
    source: &str,
    target: &Path,
    branch: Option<&str>,
    proxy: &str,
    cancellation: &TaskCancellation,
    progress_events: Option<EventSender>,
) -> Result<RepoSyncOutcome> {
    let config_overrides: Vec<String> = match normalize_proxy_url(proxy)? {
        Some(proxy) => vec![
            format!("http.proxy={proxy}"),
            format!("gitoxide.https.proxy={proxy}"),
        ],
        None => Vec::new(),
    };

    let mut prepare = gix::clone::PrepareFetch::new(
        source,
        target,
        gix::create::Kind::WithWorktree,
        gix::create::Options {
            destination_must_be_empty: true,
            ..Default::default()
        },
        gix::open::Options::isolated()
            .permissions(gix::open::Permissions::all())
            .config_overrides(config_overrides),
    )
    .map_err(|err| CoreError::git("prepare clone", err))?;

    if let Some(branch) = branch.filter(|branch| !branch.trim().is_empty()) {
        prepare = prepare
            .with_ref_name(Some(branch))
            .map_err(|err| CoreError::git("set clone branch", err))?;
    }

    let mut fetch_progress = DynNestedProgressToNestedProgress(GixProgress::new_root(
        progress_events,
        format!("clone {source}"),
    ));
    let should_interrupt = cancellation.interrupt_flag();

    let (mut prepare_checkout, _outcome) = prepare
        .fetch_then_checkout(&mut fetch_progress, &should_interrupt)
        .map_err(|err| {
            if cancellation.is_interrupted() {
                return CoreError::Cancelled;
            }
            CoreError::git("fetch clone", err)
        })?;

    let mut checkout_progress = fetch_progress.add_child("checkout");
    let (repo, _outcome) = prepare_checkout
        .main_worktree(&mut checkout_progress, &should_interrupt)
        .map_err(|err| {
            if cancellation.is_interrupted() {
                return CoreError::Cancelled;
            }
            CoreError::git("checkout worktree", err)
        })?;

    let head_commit = repo.head_id().ok().map(|id| id.to_string());
    let head_branch = repo.head().ok().and_then(|head| {
        if head.is_detached() {
            None
        } else {
            head.referent_name().map(|name| name.shorten().to_string())
        }
    });

    if cancellation.is_interrupted() {
        return Err(CoreError::Cancelled);
    }

    Ok(RepoSyncOutcome {
        head_commit,
        head_branch,
    })
}

/// Clone a git repository, publishing progress events.
pub async fn clone_repo(
    source: &str,
    target: &Path,
    branch: Option<&str>,
    proxy: &str,
    cancel: Option<&CancellationToken>,
    events: Option<&EventSender>,
) -> Result<RepoSyncOutcome> {
    let source = source.to_string();
    let source_for_log = source.clone();
    let target = PathBuf::from(target);
    let target_for_log = target.clone();
    let branch = branch.map(ToString::to_string);
    let proxy = proxy.to_string();
    let cancellation = TaskCancellation::new(cancel.cloned());

    tracing::info!(source = source.as_str(), target = %target.display(), "Starting repository sync");
    tracing::info!(source = source.as_str(), "Fetching repository contents");

    cancellation.check()?;

    let cancel_task = cancellation.spawn_interrupt_task();

    let blocking_cancellation = cancellation.clone();
    let events_clone = events.cloned();
    let task = tokio::task::spawn_blocking(move || {
        clone_repo_blocking(
            &source,
            &target,
            branch.as_deref(),
            &proxy,
            &blocking_cancellation,
            events_clone,
        )
    });

    let outcome = task.await
        .map_err(|err| CoreError::task("join", err))??;

    if let Some(cancel_task) = cancel_task {
        cancel_task.abort();
    }

    tracing::info!(source = source_for_log.as_str(), "Fetched repository contents");
    if let Some(head_branch) = &outcome.head_branch {
        tracing::info!(branch = head_branch.as_str(), "Checked out branch");
    }
    if let Some(head_commit) = &outcome.head_commit {
        tracing::info!(commit = head_commit.as_str(), "HEAD at commit");
    }
    tracing::info!(target = %target_for_log.display(), "Completed repository sync");

    Ok(outcome)
}
