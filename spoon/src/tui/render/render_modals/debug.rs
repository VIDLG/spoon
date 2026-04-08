use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Clear, Widget};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerSmartWidget};

use crate::tui::layout::{self, ModalLayoutKind};
use crate::tui::render::render_shared::{
    compose_hint_line, dim_backdrop, modal_content_layout, modal_frame, render_status_hint,
};
use crate::tui::{DebugLogState, theme};

pub(super) fn render_debug_log_modal(
    buf: &mut Buffer,
    area: Rect,
    debug: &DebugLogState,
    transient_hint: Option<&str>,
) {
    let popup = layout::modal_rect(area, layout::modal_size_for(ModalLayoutKind::DebugLog));
    let hint = compose_hint_line(transient_hint);
    dim_backdrop(buf, area);
    Clear.render(popup, buf);
    let inner = modal_frame(buf, popup, "Debug Log", theme::ACCENT_WARN);
    let [_header, content, status_row, hint_row, keys_row] = modal_content_layout(
        inner,
        ModalLayoutKind::DebugLog,
        layout::modal_content_min_height_for(popup, ModalLayoutKind::DebugLog),
    );

    TuiLoggerSmartWidget::default()
        .state(&debug.widget_state)
        .output_timestamp(Some("%H:%M:%S".to_string()))
        .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
        .output_file(false)
        .output_line(false)
        .style(theme::text_primary().bg(theme::SURFACE))
        .border_style(theme::text_muted())
        .highlight_style(theme::selected_row())
        .style_error(Style::default().fg(theme::ACCENT_DANGER))
        .style_warn(Style::default().fg(theme::ACCENT_WARN))
        .style_info(Style::default().fg(theme::ACCENT_OK))
        .style_debug(theme::text_primary())
        .style_trace(theme::text_muted())
        .render(content, buf);

    render_status_hint(
        buf,
        status_row,
        hint_row,
        keys_row,
        "interactive log viewer",
        hint,
        "Up/Down move | Left/Right view level | +/- capture level | h hide targets | f focus target | Space hide off | PgUp/PgDn page | Esc close",
    );
}
