mod action_flow;
mod animation;
mod background;
mod help;
mod keys;
mod layout;
mod render;
mod state;
pub mod test_support;
mod theme;

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crate::actions::ToolAction;
use crate::status::{self, ToolStatus};
use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::DefaultTerminal;
use tui_logger::LevelFilter;

pub(crate) use state::{
    ActionOutcome, App, AppConfigSnapshot, BackgroundAction, BackgroundEvent, BgStatusUpdate,
    ConfigKind, ConfigMenuAction, DebugLogState, EditorSetupState, FormState, HelpState, Modal,
    OutputState, Screen, ToolDetailState, ToolManagerState, ToolsKeyOutcome, Transition,
    TransitionCache, TransitionDirection, cached_screen_for_top_page, config_menu_items,
    new_config_menu, remember_screen, screen_name, start_page_transition, top_page_index,
};

pub fn run_tui(default_install_root: Option<PathBuf>, repo_root: PathBuf) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let _terminal_guard = TerminalRestoreGuard;
    let mut terminal = ratatui::init();
    terminal.hide_cursor()?;

    run_app(terminal, default_install_root, repo_root)
}

fn run_app(
    mut terminal: DefaultTerminal,
    install_root: Option<PathBuf>,
    repo_root: PathBuf,
) -> Result<()> {
    let mut app = App::new(install_root, repo_root);

    loop {
        background::poll_background_action(&mut app);
        background::poll_bg_status(&mut app);
        background::poll_transition(&mut app);
        terminal.draw(|frame| render::render(frame, &mut app))?;
        terminal.hide_cursor()?;
        let delay = if app.transition.is_some() {
            animation::ANIMATION_POLL_MS
        } else {
            animation::IDLE_POLL_MS
        };
        if event::poll(Duration::from_millis(delay))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    if keys::handle_key(&mut app, key)? {
                        break;
                    }
                }
                Event::Mouse(_) => {}
                _ => {}
            }
        }
    }

    Ok(())
}

pub(crate) fn request_bg_status_check(app: &mut App) {
    if app.transition.is_some() {
        app.pending_status_refresh = true;
        if app.status_hint.is_none() {
            app.status_hint = Some("Refreshing tool status...".to_string());
        }
    } else {
        background::start_bg_status_check(app);
    }
}

pub(crate) fn open_modal(app: &mut App, modal: Modal) {
    app.modal = Some(modal);
}

pub(crate) fn new_debug_log_modal() -> Modal {
    Modal::DebugLog(DebugLogState {
        widget_state: tui_logger::TuiWidgetState::new()
            .set_default_display_level(LevelFilter::Trace),
        follow_up: None,
    })
}

pub(crate) fn close_modal(app: &mut App) {
    app.modal = None;
}

pub(crate) fn is_actionable(
    status: &ToolStatus,
    statuses: &[ToolStatus],
    action: ToolAction,
) -> bool {
    status::action_policy(status, statuses).allows(action)
}

pub(crate) fn action_title(action: ToolAction) -> &'static str {
    match action {
        ToolAction::Install => "Install",
        ToolAction::Update => "Update",
        ToolAction::Uninstall => "Uninstall",
    }
}

pub(crate) fn action_past(action: ToolAction) -> &'static str {
    match action {
        ToolAction::Install => "installed",
        ToolAction::Update => "updated",
        ToolAction::Uninstall => "uninstalled",
    }
}

struct TerminalRestoreGuard;

impl Drop for TerminalRestoreGuard {
    fn drop(&mut self) {
        let _ = ratatui::restore();
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
    }
}
