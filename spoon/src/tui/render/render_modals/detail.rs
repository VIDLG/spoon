use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph, Widget, Wrap};

use crate::tui::layout::{self, ModalLayoutKind};
use crate::tui::{Screen, ToolDetailState, theme};
use crate::view::{ToolDetailRow, ToolDetailValueKind};

use super::super::render_shared::{dim_backdrop, modal_frame};

pub(super) fn render_tool_detail_modal(
    buf: &mut Buffer,
    area: Rect,
    _screen: &Screen,
    detail: &ToolDetailState,
) {
    let popup = layout::modal_rect(area, layout::modal_size_for(ModalLayoutKind::ToolDetail));
    dim_backdrop(buf, area);
    Clear.render(popup, buf);
    let inner = modal_frame(buf, popup, "Tool Detail", theme::ACCENT_WARN);
    let body = layout::modal_body_rect_with_title_gap(inner);
    let lines = detail.model.rows.iter().map(detail_row).collect::<Vec<_>>();
    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((detail.scroll, 0))
        .style(theme::text_primary().bg(theme::SURFACE));
    paragraph.render(body, buf);
}

fn detail_row(row: &ToolDetailRow) -> Line<'static> {
    match row {
        ToolDetailRow::Title { text } => Line::from(Span::styled(
            text.clone(),
            theme::text_primary().add_modifier(Modifier::BOLD),
        )),
        ToolDetailRow::ActionGroup {
            install,
            update,
            uninstall,
        } => operation_line(*install, *update, *uninstall),
        ToolDetailRow::Field {
            label,
            value,
            value_kind,
        } => field_line(label, value, *value_kind),
    }
}

fn field_line(label: &str, value: &str, value_kind: ToolDetailValueKind) -> Line<'static> {
    let value_style = match value_kind {
        ToolDetailValueKind::Package => theme::field_package(),
        ToolDetailValueKind::Backend => theme::field_backend(),
        ToolDetailValueKind::Path => theme::field_path(),
        ToolDetailValueKind::Version => theme::field_version(),
        ToolDetailValueKind::State => Style::default().fg(theme::status_tone(value)),
        ToolDetailValueKind::Default => theme::text_primary(),
    };

    Line::from(vec![
        Span::styled(
            format!("{label}: "),
            theme::text_muted().add_modifier(Modifier::BOLD),
        ),
        Span::styled(value.to_string(), value_style),
    ])
}

fn operation_line(install_on: bool, update_on: bool, uninstall_on: bool) -> Line<'static> {
    let op_style = |enabled: bool, color| {
        if enabled {
            Style::default().fg(color).add_modifier(Modifier::BOLD)
        } else {
            theme::text_muted()
        }
    };

    Line::from(vec![
        Span::styled(
            "available operations: ",
            theme::text_muted().add_modifier(Modifier::BOLD),
        ),
        Span::styled("i", op_style(install_on, theme::ACCENT_WARN)),
        Span::raw(" "),
        Span::styled("u", op_style(update_on, theme::ACCENT_OK)),
        Span::raw(" "),
        Span::styled("x", op_style(uninstall_on, theme::ACCENT_DANGER)),
    ])
}

#[cfg(test)]
mod tests {
    use crate::view::{ToolDetailRow, ToolDetailValueKind, tool_detail_plain_lines};

    use super::detail_row;

    #[test]
    fn ops_line_renders_compact_tokens() {
        let line = detail_row(&ToolDetailRow::ActionGroup {
            install: true,
            update: false,
            uninstall: true,
        });
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();
        assert_eq!(rendered, "available operations: i u x");
    }

    #[test]
    fn plain_lines_keep_existing_tool_detail_text_shape() {
        let lines = tool_detail_plain_lines(&crate::view::ToolDetailModel {
            title: "Example".to_string(),
            rows: vec![
                ToolDetailRow::Title {
                    text: "Example".to_string(),
                },
                ToolDetailRow::Field {
                    label: "summary".to_string(),
                    value: "hello".to_string(),
                    value_kind: ToolDetailValueKind::Default,
                },
                ToolDetailRow::ActionGroup {
                    install: true,
                    update: false,
                    uninstall: true,
                },
            ],
        });
        assert_eq!(lines[0], "Example");
        assert_eq!(lines[1], "summary: hello");
        assert_eq!(lines[2], "available operations: i=on u=off x=on");
    }
}
