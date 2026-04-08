use std::path::{Path, PathBuf};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{ListState, TableState};
use tokio::sync::mpsc::UnboundedReceiver;
use tui_logger::{LevelFilter, TuiWidgetState};

use crate::actions::ToolAction;
use crate::editor;
use crate::packages;
use crate::runtime;
use crate::status::{self, ToolStatus};
use crate::view::ToolDetailModel;

use super::animation;

pub(crate) struct App {
    pub(crate) install_root: Option<PathBuf>,
    pub(crate) repo_root: PathBuf,
    pub(crate) last_frame_area: Rect,
    pub(crate) statuses_snapshot: Vec<ToolStatus>,
    pub(crate) config_snapshot: AppConfigSnapshot,
    pub(crate) screen: Screen,
    pub(crate) saved_config_screen: Option<Screen>,
    pub(crate) saved_tools_screen: Option<Screen>,
    pub(crate) modal: Option<Modal>,
    pub(crate) transition: Option<Transition>,
    pub(crate) background_action: Option<BackgroundAction>,
    pub(crate) status_hint: Option<String>,
    pub(crate) pending_status_refresh: bool,
    pub(crate) bg_status_rx: Option<UnboundedReceiver<BgStatusUpdate>>,
}

#[derive(Clone)]
pub(crate) enum Screen {
    Tools(ToolManagerState),
    ConfigMenu { state: ListState },
}

#[derive(Clone)]
pub(crate) enum Modal {
    ToolDetail(ToolDetailState),
    Form(FormState),
    EditorSetup(EditorSetupState),
    Output(OutputState),
    CancelRunningConfirm(CancelRunningConfirmState),
    DebugLog(DebugLogState),
    Help(HelpState),
    QuitConfirm,
}

#[derive(Clone)]
pub(crate) struct ToolDetailState {
    pub(crate) scroll: u16,
    pub(crate) model: ToolDetailModel,
}

#[derive(Clone)]
pub(crate) struct OutputState {
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) lines: Vec<String>,
    pub(crate) scroll: u16,
    pub(crate) max_scroll: u16,
    pub(crate) page_step: u16,
    pub(crate) auto_scroll: bool,
    pub(crate) snap_to_bottom_on_render: bool,
    pub(crate) running: bool,
    pub(crate) follow_up: Option<Box<Modal>>,
}

#[derive(Clone)]
pub(crate) struct CancelRunningConfirmState {
    pub(crate) quit_after_cancel: bool,
    pub(crate) follow_up: Option<Box<Modal>>,
}

#[derive(Clone)]
pub(crate) struct HelpState {
    pub(crate) title: String,
    pub(crate) lines: Vec<String>,
    pub(crate) scroll: u16,
    pub(crate) follow_up: Option<Box<Modal>>,
}

pub(crate) struct DebugLogState {
    pub(crate) widget_state: TuiWidgetState,
    pub(crate) follow_up: Option<Box<Modal>>,
}

impl Clone for DebugLogState {
    fn clone(&self) -> Self {
        Self {
            widget_state: TuiWidgetState::new().set_default_display_level(LevelFilter::Trace),
            follow_up: self.follow_up.clone(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct EditorSetupState {
    pub(crate) selected: usize,
    pub(crate) current_command: String,
    pub(crate) requested_kind: Option<ConfigKind>,
    pub(crate) notice: Option<String>,
}

pub(crate) struct BackgroundAction {
    pub(crate) rx: UnboundedReceiver<BackgroundEvent>,
    pub(crate) cancel: crate::bridge::CancellationToken,
}

pub(crate) struct ActionOutcome {
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) lines: Vec<String>,
    pub(crate) append_lines: bool,
    pub(crate) follow_up: Option<Box<Modal>>,
}

pub(crate) enum BackgroundEvent {
    AppendLine(String),
    ReplaceLastLine(String),
    Complete(ActionOutcome),
}

#[derive(Clone)]
pub(crate) struct ToolManagerState {
    pub(crate) table_state: TableState,
    pub(crate) selected: Vec<bool>,
    pub(crate) statuses: Vec<ToolStatus>,
}

pub(crate) struct ToolsKeyOutcome {
    pub(crate) next_modal: Option<Modal>,
    pub(crate) next_screen: Option<Screen>,
    pub(crate) background_action: Option<BackgroundAction>,
    pub(crate) next_hint: Option<String>,
    pub(crate) request_status_refresh: bool,
}

#[derive(Clone)]
pub(crate) enum BgStatusUpdate {
    Config(AppConfigSnapshot),
    Statuses(Vec<ToolStatus>),
}

#[derive(Clone)]
pub(crate) struct AppConfigSnapshot {
    pub(crate) editor_command: String,
    pub(crate) editor_available: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ConfigKind {
    Global,
    Package(&'static str),
}

#[derive(Clone)]
pub(crate) struct FormState {
    pub(crate) kind: ConfigKind,
    pub(crate) title: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) enum ConfigMenuAction {
    Editor,
    Global,
    Package(&'static str),
}

#[derive(Clone)]
pub(crate) struct Transition {
    pub(crate) from: Screen,
    pub(crate) direction: TransitionDirection,
    pub(crate) step: u16,
    pub(crate) steps: u16,
    pub(crate) cache: Option<TransitionCache>,
}

#[derive(Clone)]
pub(crate) struct TransitionCache {
    pub(crate) area: Rect,
    pub(crate) from_buf: Buffer,
    pub(crate) to_buf: Buffer,
}

#[derive(Clone, Copy)]
pub(crate) enum TransitionDirection {
    Forward,
    Backward,
}

impl App {
    pub(crate) fn new(install_root: Option<PathBuf>, repo_root: PathBuf) -> Self {
        let initial_statuses = status::collect_statuses_fast(install_root.as_deref());
        let initial_config = AppConfigSnapshot::load();
        let bg_root = install_root.clone();
        let rx = runtime::spawn_with_sender(move |tx| {
            let _ = tx.send(BgStatusUpdate::Config(AppConfigSnapshot::load()));
            let full = status::collect_statuses(bg_root.as_deref());
            let _ = tx.send(BgStatusUpdate::Statuses(full));
        });

        Self {
            install_root: install_root.clone(),
            repo_root,
            last_frame_area: Rect::new(0, 0, 0, 0),
            statuses_snapshot: initial_statuses,
            config_snapshot: initial_config,
            screen: new_config_menu(),
            saved_config_screen: None,
            saved_tools_screen: None,
            modal: None,
            transition: None,
            background_action: None,
            status_hint: Some("Loading tool versions...".to_string()),
            pending_status_refresh: false,
            bg_status_rx: Some(rx),
        }
    }
}

impl AppConfigSnapshot {
    pub(crate) fn load() -> Self {
        let editor_status = editor::default_editor_status();
        Self {
            editor_command: editor_status.command,
            editor_available: editor_status.available,
        }
    }
}

impl FormState {
    pub(crate) fn from_global() -> Self {
        Self {
            kind: ConfigKind::Global,
            title: "Global",
        }
    }

    pub(crate) fn from_package(package_key: &'static str, title: &'static str) -> Self {
        Self {
            kind: ConfigKind::Package(package_key),
            title,
        }
    }
}

impl ToolManagerState {
    pub(crate) fn from_statuses(statuses: Vec<ToolStatus>) -> Self {
        let mut statuses = statuses;
        sort_statuses_for_tools_table(&mut statuses);
        let mut state = TableState::default();
        state.select((!statuses.is_empty()).then_some(0));
        Self {
            table_state: state,
            selected: vec![false; statuses.len()],
            statuses,
        }
    }

    pub(crate) fn apply_statuses(&mut self, new_statuses: Vec<ToolStatus>) {
        self.statuses = new_statuses;
        sort_statuses_for_tools_table(&mut self.statuses);
        self.selected.resize(self.statuses.len(), false);
        let selected = self.table_state.selected().unwrap_or(0);
        if self.statuses.is_empty() {
            self.table_state.select(None);
        } else if selected >= self.statuses.len() {
            self.table_state.select(Some(self.statuses.len() - 1));
        }
    }

    pub(crate) fn refresh_fast(&mut self, install_root: Option<&Path>) {
        self.apply_statuses(status::collect_statuses_fast(install_root));
    }

    pub(crate) fn toggle_select_all(&mut self) {
        let should_select_all = self.selected.iter().any(|flag| !*flag);
        self.selected
            .iter_mut()
            .for_each(|flag| *flag = should_select_all);
    }

    pub(crate) fn select_installable(&mut self) {
        let all_installable_selected = self
            .statuses
            .iter()
            .zip(self.selected.iter())
            .filter(|(status, _)| super::is_actionable(status, &self.statuses, ToolAction::Install))
            .all(|(_, selected)| *selected);
        let should_select = !all_installable_selected;
        for (flag, status) in self.selected.iter_mut().zip(self.statuses.iter()) {
            *flag =
                should_select && super::is_actionable(status, &self.statuses, ToolAction::Install);
        }
    }

    pub(crate) fn select_installed(&mut self) {
        let all_installed_selected = self
            .statuses
            .iter()
            .zip(self.selected.iter())
            .filter(|(status, _)| status.is_detected())
            .all(|(_, selected)| *selected);
        let should_select = !all_installed_selected;
        for (flag, status) in self.selected.iter_mut().zip(self.statuses.iter()) {
            *flag = should_select && status.is_detected();
        }
    }

    pub(crate) fn selected_tools_for_action(
        &self,
        action: ToolAction,
    ) -> Vec<&'static crate::packages::tool::Tool> {
        let selected: Vec<&'static crate::packages::tool::Tool> = self
            .statuses
            .iter()
            .zip(self.selected.iter())
            .filter_map(|(status, enabled)| {
                if !super::is_actionable(status, &self.statuses, action) || !enabled {
                    return None;
                }
                Some(status.tool)
            })
            .collect();

        if !selected.is_empty() {
            return selected;
        }

        self.table_state
            .selected()
            .and_then(|index| self.statuses.get(index))
            .filter(|status| super::is_actionable(status, &self.statuses, action))
            .map(|status| vec![status.tool])
            .unwrap_or_default()
    }

    pub(crate) fn selected_index(&self) -> Option<usize> {
        self.table_state.selected()
    }
}

fn sort_statuses_for_tools_table(statuses: &mut [ToolStatus]) {
    statuses.sort_by_key(|status| crate::packages::tool::tool_sort_key(status.tool));
}

pub(crate) fn new_config_menu() -> Screen {
    let mut state = ListState::default();
    state.select(Some(0));
    Screen::ConfigMenu { state }
}

pub(crate) fn top_page_index(screen: &Screen) -> usize {
    match screen {
        Screen::ConfigMenu { .. } => 0,
        Screen::Tools(_) => 1,
    }
}

pub(crate) fn screen_for_top_page(
    index: usize,
    _install_root: Option<&Path>,
    statuses_snapshot: &[ToolStatus],
    repo_root: &Path,
) -> Screen {
    let _ = repo_root;
    match index % 2 {
        0 => new_config_menu(),
        _ => Screen::Tools(ToolManagerState::from_statuses(statuses_snapshot.to_vec())),
    }
}

pub(crate) fn screen_name(screen: &Screen) -> &'static str {
    match screen {
        Screen::Tools(_) => "Tools",
        Screen::ConfigMenu { .. } => "Configure",
    }
}

pub(crate) fn config_menu_items() -> Vec<(&'static str, ConfigMenuAction)> {
    let mut items = vec![
        ("Editor Setup", ConfigMenuAction::Editor),
        ("Global settings (proxy/root)", ConfigMenuAction::Global),
    ];
    items.extend(
        packages::config_target_descriptors()
            .into_iter()
            .map(|descriptor| {
                (
                    descriptor.menu_label,
                    ConfigMenuAction::Package(descriptor.package_key),
                )
            }),
    );
    items
}

pub(crate) fn start_page_transition(
    app: &mut App,
    next_screen: Screen,
    direction: TransitionDirection,
) {
    let from = app.screen.clone();
    app.screen = next_screen;
    app.transition = Some(Transition {
        from,
        direction,
        step: 0,
        steps: animation::PAGE_TRANSITION_STEPS,
        cache: None,
    });
}

pub(crate) fn remember_screen(app: &mut App, screen: &Screen) {
    match screen {
        Screen::ConfigMenu { .. } => app.saved_config_screen = Some(screen.clone()),
        Screen::Tools(_) => app.saved_tools_screen = Some(screen.clone()),
    }
}

pub(crate) fn cached_screen_for_top_page(
    app: &App,
    index: usize,
    install_root: Option<&Path>,
    repo_root: &Path,
) -> Screen {
    match index {
        0 => app
            .saved_config_screen
            .as_ref()
            .cloned()
            .unwrap_or_else(new_config_menu),
        1 => app.saved_tools_screen.as_ref().cloned().unwrap_or_else(|| {
            screen_for_top_page(index, install_root, &app.statuses_snapshot, repo_root)
        }),
        _ => screen_for_top_page(index, install_root, &app.statuses_snapshot, repo_root),
    }
}
