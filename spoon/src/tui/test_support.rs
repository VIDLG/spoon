use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, Once, OnceLock};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

use crate::clipboard;
use crate::config;
use crate::editor;
use crate::launcher;
use crate::service::scoop;
use crate::status::ToolStatus;

use super::{App, Modal, OutputState, Screen, background, keys};

static TEST_MODE: Once = Once::new();
static TEST_HOME_COUNTER: AtomicUsize = AtomicUsize::new(0);
static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub struct Harness {
    app: App,
    _guard: MutexGuard<'static, ()>,
}

impl Default for Harness {
    fn default() -> Self {
        Self::new()
    }
}

impl Harness {
    pub fn new() -> Self {
        TEST_MODE.call_once(|| {
            clipboard::enable_test_mode();
            config::enable_test_mode();
            editor::enable_test_mode();
            launcher::enable_test_mode();
        });
        editor::reset_availability_overrides();
        editor::set_test_candidate_availability(None);
        Self::with_install_root(None)
    }

    pub fn with_install_root(install_root: Option<PathBuf>) -> Self {
        TEST_MODE.call_once(|| {
            clipboard::enable_test_mode();
            config::enable_test_mode();
            editor::enable_test_mode();
            launcher::enable_test_mode();
        });
        editor::reset_availability_overrides();
        editor::set_test_candidate_availability(None);
        let guard = TEST_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let test_home = std::env::temp_dir().join(format!(
            "spoon-test-home-{}",
            TEST_HOME_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&test_home);
        std::fs::create_dir_all(&test_home).expect("create test home");
        config::set_home_override(test_home);
        Self {
            app: App::new(
                install_root,
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .expect("repo root")
                    .to_path_buf(),
            ),
            _guard: guard,
        }
    }

    pub fn enable_real_scoop_backend(&self) {
        scoop::set_real_backend_test_mode(true);
    }

    pub fn disable_real_scoop_backend(&self) {
        scoop::set_real_backend_test_mode(false);
    }

    pub fn press(&mut self, code: KeyCode) -> Result<bool> {
        let quit = keys::handle_key(&mut self.app, KeyEvent::new(code, KeyModifiers::NONE))?;
        self.settle();
        Ok(quit)
    }

    pub fn press_without_settle(&mut self, code: KeyCode) -> Result<bool> {
        keys::handle_key(&mut self.app, KeyEvent::new(code, KeyModifiers::NONE))
    }

    pub fn mouse_scroll_down(&mut self) -> Result<()> {
        keys::handle_mouse(
            &mut self.app,
            MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column: 0,
                row: 0,
                modifiers: KeyModifiers::NONE,
            },
        )?;
        self.settle();
        Ok(())
    }

    pub fn mouse_scroll_up(&mut self) -> Result<()> {
        keys::handle_mouse(
            &mut self.app,
            MouseEvent {
                kind: MouseEventKind::ScrollUp,
                column: 0,
                row: 0,
                modifiers: KeyModifiers::NONE,
            },
        )?;
        self.settle();
        Ok(())
    }

    pub fn mouse_left_click(&mut self, column: u16, row: u16) -> Result<()> {
        self.render_text(140, 40);
        keys::handle_mouse(
            &mut self.app,
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            },
        )?;
        self.settle();
        Ok(())
    }

    pub fn screen_name(&self) -> &'static str {
        match &self.app.screen {
            Screen::ConfigMenu { .. } => "Configure",
            Screen::Tools(_) => "Tools",
        }
    }

    pub fn modal_name(&self) -> Option<&'static str> {
        match self.app.modal.as_ref() {
            Some(Modal::ToolDetail(_)) => Some("ToolDetail"),
            Some(Modal::Form(_)) => Some("Configuration"),
            Some(Modal::EditorSetup(_)) => Some("EditorSetup"),
            Some(Modal::Output(_)) => Some("Output"),
            Some(Modal::CancelRunningConfirm(_)) => Some("CancelRunningConfirm"),
            Some(Modal::DebugLog(_)) => Some("DebugLog"),
            Some(Modal::Help(_)) => Some("Help"),
            Some(Modal::QuitConfirm) => Some("QuitConfirm"),
            None => None,
        }
    }

    pub fn form_title(&self) -> Option<&'static str> {
        match self.app.modal.as_ref() {
            Some(Modal::Form(form)) => Some(form.title),
            _ => None,
        }
    }

    pub fn selected_tool_marked(&self) -> Option<bool> {
        match &self.app.screen {
            Screen::Tools(state) => state
                .table_state
                .selected()
                .and_then(|index| state.selected.get(index).copied()),
            _ => None,
        }
    }

    pub fn selected_tool_count(&self) -> Option<usize> {
        match &self.app.screen {
            Screen::Tools(state) => Some(state.selected.iter().filter(|flag| **flag).count()),
            _ => None,
        }
    }

    pub fn tools_selected_index(&self) -> Option<usize> {
        match &self.app.screen {
            Screen::Tools(state) => state.table_state.selected(),
            _ => None,
        }
    }

    pub fn config_selected_index(&self) -> Option<usize> {
        match &self.app.screen {
            Screen::ConfigMenu { state } => state.selected(),
            _ => None,
        }
    }

    pub fn output_title(&self) -> Option<String> {
        match self.app.modal.as_ref() {
            Some(Modal::Output(output)) => Some(output.title.clone()),
            _ => None,
        }
    }

    pub fn output_lines(&self) -> Option<Vec<String>> {
        match self.app.modal.as_ref() {
            Some(Modal::Output(output)) => Some(output.lines.clone()),
            _ => None,
        }
    }

    pub fn output_running(&self) -> Option<bool> {
        match self.app.modal.as_ref() {
            Some(Modal::Output(output)) => Some(output.running),
            _ => None,
        }
    }

    pub fn output_scroll(&self) -> Option<u16> {
        match self.app.modal.as_ref() {
            Some(Modal::Output(output)) => Some(output.scroll),
            _ => None,
        }
    }

    pub fn set_output_modal_for_test(&mut self, title: &str, lines: Vec<String>, running: bool) {
        self.app.modal = Some(Modal::Output(OutputState {
            title: title.to_string(),
            status: if running {
                "running command".to_string()
            } else {
                "action completed".to_string()
            },
            lines,
            scroll: 0,
            max_scroll: 0,
            page_step: 10,
            auto_scroll: false,
            snap_to_bottom_on_render: false,
            running,
            follow_up: None,
        }));
    }

    pub fn set_output_modal_with_state_for_test(
        &mut self,
        title: &str,
        lines: Vec<String>,
        running: bool,
        auto_scroll: bool,
        scroll: u16,
    ) {
        self.app.modal = Some(Modal::Output(OutputState {
            title: title.to_string(),
            status: if running {
                "running command".to_string()
            } else {
                "action completed".to_string()
            },
            lines,
            scroll,
            max_scroll: 0,
            page_step: 10,
            auto_scroll,
            snap_to_bottom_on_render: false,
            running,
            follow_up: None,
        }));
    }

    pub fn set_output_scroll_metrics_for_test(&mut self, max_scroll: u16, page_step: u16) {
        if let Some(Modal::Output(output)) = self.app.modal.as_mut() {
            output.max_scroll = max_scroll;
            output.page_step = page_step.max(1);
        }
    }

    pub fn append_output_line_for_test(&mut self, line: impl Into<String>) {
        if let Some(Modal::Output(output)) = self.app.modal.as_mut() {
            output.lines.push(line.into());
            if output.auto_scroll {
                output.snap_to_bottom_on_render = true;
            }
        }
    }

    pub fn replace_output_last_line_for_test(&mut self, line: impl Into<String>) {
        if let Some(Modal::Output(output)) = self.app.modal.as_mut() {
            let line = line.into();
            if let Some(last) = output.lines.last_mut() {
                *last = line;
            } else {
                output.lines.push(line);
            }
            if output.auto_scroll {
                output.snap_to_bottom_on_render = true;
            }
        }
    }

    pub fn complete_output_for_test(
        &mut self,
        status: impl Into<String>,
        lines: Vec<String>,
        append_lines: bool,
    ) {
        if let Some(Modal::Output(output)) = self.app.modal.as_mut() {
            let was_auto_scroll = output.auto_scroll;
            output.status = status.into();
            if append_lines {
                output.lines.extend(lines);
            } else {
                output.lines = lines;
            }
            output.running = false;
            output.auto_scroll = false;
            if was_auto_scroll {
                output.snap_to_bottom_on_render = true;
            }
        }
    }

    pub fn status_hint(&self) -> Option<String> {
        self.app.status_hint.clone()
    }

    pub fn selected_tool_key(&self) -> Option<&'static str> {
        match &self.app.screen {
            Screen::Tools(state) => state
                .table_state
                .selected()
                .and_then(|index| state.statuses.get(index))
                .map(|status| status.tool.key),
            _ => None,
        }
    }

    pub fn selected_tool_detected(&self) -> Option<bool> {
        match &self.app.screen {
            Screen::Tools(state) => state
                .table_state
                .selected()
                .and_then(|index| state.statuses.get(index))
                .map(|status| status.is_detected()),
            _ => None,
        }
    }

    pub fn selected_tool_installed_size_bytes(&self) -> Option<Option<u64>> {
        match &self.app.screen {
            Screen::Tools(state) => state
                .table_state
                .selected()
                .and_then(|index| state.statuses.get(index))
                .map(|status| status.installed_size_bytes),
            _ => None,
        }
    }

    pub fn tool_installed_size_bytes(&self, key: &str) -> Option<Option<u64>> {
        match &self.app.screen {
            Screen::Tools(state) => state
                .statuses
                .iter()
                .find(|status| status.tool.key == key)
                .map(|status| status.installed_size_bytes),
            _ => None,
        }
    }

    pub fn tool_version(&self, key: &str) -> Option<Option<String>> {
        match &self.app.screen {
            Screen::Tools(state) => state
                .statuses
                .iter()
                .find(|status| status.tool.key == key)
                .map(|status| status.version.clone()),
            _ => None,
        }
    }

    pub fn set_tool_statuses_for_test(&mut self, statuses: Vec<ToolStatus>) {
        self.app.statuses_snapshot = statuses.clone();
        if let Screen::Tools(state) = &mut self.app.screen {
            state.apply_statuses(statuses);
        }
    }

    pub fn wait_until<F>(&mut self, timeout: Duration, mut predicate: F) -> bool
    where
        F: FnMut(&Self) -> bool,
    {
        let start = Instant::now();
        while start.elapsed() < timeout {
            self.settle();
            if predicate(self) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        predicate(self)
    }

    pub fn render_text(&mut self, width: u16, height: u16) -> Vec<String> {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("create test terminal");
        terminal
            .draw(|frame| super::render::render(frame, &mut self.app))
            .expect("render test frame");
        let buffer = terminal.backend().buffer().clone();
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect()
    }

    pub fn render_cell_symbols(&mut self, width: u16, height: u16) -> Vec<Vec<String>> {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("create test terminal");
        terminal
            .draw(|frame| super::render::render(frame, &mut self.app))
            .expect("render test frame");
        let buffer = terminal.backend().buffer().clone();
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol().to_string())
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn clipboard_text(&self) -> Option<String> {
        clipboard::test_contents()
    }

    pub fn refresh_config_snapshot_for_test(&mut self) {
        self.app.config_snapshot = super::AppConfigSnapshot::load();
    }

    fn settle(&mut self) {
        for _ in 0..64 {
            let had_transition = self.app.transition.is_some();
            let had_background = self.app.background_action.is_some();
            let had_status = self.app.bg_status_rx.is_some();

            if had_transition {
                background::poll_transition(&mut self.app);
            }
            if had_background {
                background::poll_background_action(&mut self.app);
            }
            if had_status {
                background::poll_bg_status(&mut self.app);
            }

            if !had_transition && !had_background && !had_status {
                break;
            }

            std::thread::yield_now();
        }
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        scoop::set_real_backend_test_mode(false);
    }
}
