mod modal;
mod navigation;
mod screens;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Padding};

use super::background;
use super::help::help_lines_for_screen;
use super::{
    App, HelpState, Modal, Screen, new_debug_log_modal, open_modal, request_bg_status_check,
    screen_name,
};

pub(super) fn handle_key(app: &mut App, key: KeyEvent) -> Result<bool> {
    let code = key.code;
    if app.modal.is_some() {
        return modal::handle_modal_key(app, key);
    }

    if app.transition.is_some() {
        if matches!(code, KeyCode::Char('q') | KeyCode::Char('Q')) {
            return Ok(true);
        }
        background::finish_transition(app);
    }

    if matches!(code, KeyCode::Char('?')) {
        open_modal(
            app,
            Modal::Help(HelpState {
                title: format!("Help - {}", screen_name(&app.screen)),
                lines: help_lines_for_screen(&app.screen),
                scroll: 0,
                follow_up: None,
            }),
        );
        return Ok(false);
    }

    if matches!(code, KeyCode::Char('D')) {
        open_modal(app, new_debug_log_modal());
        return Ok(false);
    }

    if matches!(code, KeyCode::Char('q') | KeyCode::Char('Q')) {
        return Ok(true);
    }

    if matches!(app.screen, Screen::ConfigMenu { .. })
        && matches!(code, KeyCode::Char('r') | KeyCode::Char('R'))
    {
        request_bg_status_check(app);
        return Ok(false);
    }

    if navigation::handle_top_navigation(app, code) {
        return Ok(false);
    }

    let (next_screen, next_modal) = match &mut app.screen {
        Screen::Tools(state) => {
            let outcome = screens::handle_tools_key(
                state,
                code,
                app.install_root.as_deref(),
                app.background_action.is_some(),
            )?;
            if let Some(background_action) = outcome.background_action {
                app.background_action = Some(background_action);
            }
            if let Some(hint) = outcome.next_hint {
                app.status_hint = Some(hint);
            }
            if outcome.request_status_refresh {
                request_bg_status_check(app);
            }
            (outcome.next_screen, outcome.next_modal)
        }
        Screen::ConfigMenu { state } => screens::handle_config_menu_key(state, code),
    };

    if let Some(screen) = next_screen {
        navigation::apply_screen_change(app, screen);
    }
    if let Some(modal) = next_modal {
        open_modal(app, modal);
    }

    Ok(false)
}

pub(super) fn handle_mouse(app: &mut App, mouse: MouseEvent) -> Result<()> {
    if app.modal.is_some() {
        modal::handle_modal_mouse(app, mouse);
        return Ok(());
    }
    if !matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
        return Ok(());
    }
    let area = app.last_frame_area;
    if area.width == 0 || area.height == 0 {
        return Ok(());
    }
    if hit_tools_tab(area, mouse.column, mouse.row) {
        if !matches!(app.screen, Screen::Tools(_)) {
            let next = super::cached_screen_for_top_page(
                app,
                1,
                app.install_root.as_deref(),
                &app.repo_root,
            );
            navigation::apply_screen_change(app, next);
        }
        return Ok(());
    }
    if hit_config_tab(area, mouse.column, mouse.row) {
        if !matches!(app.screen, Screen::ConfigMenu { .. }) {
            navigation::apply_screen_change(app, super::new_config_menu());
        }
        return Ok(());
    }
    match &mut app.screen {
        Screen::Tools(state) => {
            if let Some(index) = hit_tools_row(area, mouse.column, mouse.row, state.statuses.len())
            {
                state.table_state.select(Some(index));
            }
        }
        Screen::ConfigMenu { state } => {
            let total = super::config_menu_items().len();
            if let Some(index) = hit_config_row(area, mouse.column, mouse.row, total) {
                state.select(Some(index));
            }
        }
    }
    Ok(())
}

fn page_body_rect(area: Rect) -> Rect {
    let summary_height = 1;
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(super::layout::page_shell_constraints(
            area,
            summary_height,
            1,
        ))
        .split(area);
    rows[3]
}

fn panel_inner(area: Rect) -> Rect {
    let (pad_x, pad_y) = super::layout::content_panel_padding_for(area);
    let block = Block::default().borders(Borders::ALL).padding(Padding {
        left: pad_x,
        right: pad_x,
        top: pad_y,
        bottom: pad_y,
    });
    block.inner(area)
}

fn hit_tools_row(area: Rect, column: u16, row: u16, total_rows: usize) -> Option<usize> {
    let body = page_body_rect(area);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(super::layout::TOOLS_TABLE_MIN_HEIGHT)])
        .split(body);
    let inner = panel_inner(rows[0]);
    if column < inner.x || column >= inner.x + inner.width || row <= inner.y {
        return None;
    }
    let index = row.saturating_sub(inner.y + 1) as usize;
    (index < total_rows).then_some(index)
}

fn hit_config_row(area: Rect, column: u16, row: u16, total_rows: usize) -> Option<usize> {
    let body = page_body_rect(area);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(super::layout::config_page_constraints(body))
        .split(body);
    let inner = panel_inner(rows[2]);
    if column < inner.x || column >= inner.x + inner.width || row < inner.y {
        return None;
    }
    let index = row.saturating_sub(inner.y) as usize;
    (index < total_rows).then_some(index)
}

fn hit_config_tab(area: Rect, column: u16, row: u16) -> bool {
    row > area.y
        && row < area.y + super::layout::TAB_BAR_HEIGHT.saturating_sub(1)
        && column >= area.x + 2
        && column <= area.x + 16
}

fn hit_tools_tab(area: Rect, column: u16, row: u16) -> bool {
    row > area.y
        && row < area.y + super::layout::TAB_BAR_HEIGHT.saturating_sub(1)
        && column >= area.x + 18
        && column <= area.x + 28
}
