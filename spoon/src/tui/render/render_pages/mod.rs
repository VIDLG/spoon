mod config;
mod tools;

use ratatui::buffer::Buffer;

use crate::status::ToolStatus;
use crate::tui::{AppConfigSnapshot, Screen};

pub(super) fn render_screen(
    buf: &mut Buffer,
    screen: &mut Screen,
    area: ratatui::layout::Rect,
    transient_hint: Option<&str>,
    statuses_snapshot: &[ToolStatus],
    config_snapshot: &AppConfigSnapshot,
) {
    match screen {
        Screen::Tools(state) => tools::render_tools(buf, state, area, transient_hint),
        Screen::ConfigMenu { state } => config::render_config_menu(
            buf,
            state,
            area,
            transient_hint,
            statuses_snapshot,
            config_snapshot,
        ),
    }
}
