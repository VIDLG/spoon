use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph, Widget, Wrap};

use crate::tui::layout::{self, ModalLayoutKind};
use crate::tui::theme;

use super::super::render_shared::{dim_backdrop, modal_frame};

pub(super) fn render_quit_modal(buf: &mut Buffer, area: Rect) {
    let popup = layout::modal_rect(area, layout::modal_size_for(ModalLayoutKind::QuitConfirm));
    dim_backdrop(buf, area);
    Clear.render(popup, buf);
    let inner = modal_frame(buf, popup, "Quit", theme::ACCENT_WARN);
    let body = layout::modal_body_rect(inner);
    Paragraph::new(vec![
        Line::from(Span::styled(
            "Exit spoon?",
            Style::default()
                .fg(theme::ACCENT_WARN)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Enter / y / q to confirm"),
        Line::from("Esc / n to stay"),
    ])
    .wrap(Wrap { trim: false })
    .render(body, buf);
}

pub(super) fn render_cancel_running_modal(buf: &mut Buffer, area: Rect) {
    let popup = layout::modal_rect(
        area,
        layout::modal_size_for(ModalLayoutKind::CancelRunningConfirm),
    );
    dim_backdrop(buf, area);
    Clear.render(popup, buf);
    let inner = modal_frame(buf, popup, "Cancel Action", theme::ACCENT_DANGER);
    let body = layout::modal_body_rect(inner);
    Paragraph::new(vec![
        Line::from(Span::styled(
            "Cancel the running action?",
            Style::default()
                .fg(theme::ACCENT_DANGER)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Enter / y / q to cancel now"),
        Line::from("Esc / n to keep it running"),
    ])
    .wrap(Wrap { trim: false })
    .render(body, buf);
}
