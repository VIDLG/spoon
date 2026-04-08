use ratatui::buffer::Buffer;
use ratatui::layout::{Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Padding, Paragraph, Row, Widget, Wrap};

use crate::tui::animation;
use crate::tui::layout;
use crate::tui::theme;

pub(super) fn transition_progress(width: u16, step: u16, steps: u16) -> u16 {
    if width == 0 || steps == 0 {
        return 0;
    }
    let eased = animation::smoothstep(step, steps);
    ((width as f32) * eased).round() as u16
}

pub(super) fn compose_hint_line(transient_hint: Option<&str>) -> Option<Line<'static>> {
    transient_hint
        .filter(|text| !text.trim().is_empty())
        .map(|text| {
            Line::from(Span::styled(
                text.to_string(),
                Style::default()
                    .fg(theme::hint_tone(text))
                    .add_modifier(Modifier::BOLD),
            ))
        })
}

pub(super) fn render_page_shell(
    buf: &mut Buffer,
    area: Rect,
    page_idx: usize,
    summary: &str,
    status: &str,
    keys: &str,
    transient_hint: Option<&str>,
) -> Rect {
    let summary_height = estimate_wrapped_lines(area.width, 0, summary).clamp(
        layout::PAGE_SUMMARY_MIN_HEIGHT as usize,
        layout::page_summary_max_height_for(area) as usize,
    ) as u16;
    let hint_line = compose_hint_line(transient_hint);
    let hint_text = hint_line
        .as_ref()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .unwrap_or_default();
    let hint_height = estimate_wrapped_lines(area.width, 5, &hint_text).clamp(
        layout::HINT_BAR_MIN_HEIGHT as usize,
        layout::hint_bar_max_height_for(area) as usize,
    ) as u16;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(layout::page_shell_constraints(
            area,
            summary_height,
            hint_height,
        ))
        .split(area);

    render_tab_bar(buf, chunks[0], page_idx);

    Paragraph::new(Line::from(Span::styled(summary, theme::text_primary())))
        .wrap(Wrap { trim: false })
        .render(chunks[1], buf);

    render_status_hint(
        buf, chunks[4], chunks[5], chunks[6], status, hint_line, keys,
    );

    chunks[3]
}

fn render_tab_bar(buf: &mut Buffer, area: Rect, page_idx: usize) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let configure_style = if page_idx == 0 {
        theme::tabs_selected()
    } else {
        theme::tabs_base()
    };
    let tools_style = if page_idx == 1 {
        theme::tabs_selected()
    } else {
        theme::tabs_base()
    };

    Paragraph::new(Line::from(vec![
        Span::styled("1 Configure", configure_style),
        Span::styled(" | ", theme::text_muted()),
        Span::styled("2 Tools", tools_style),
    ]))
    .render(
        Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        },
        buf,
    );

    if area.height < 2 {
        return;
    }

    Paragraph::new(Line::from(Span::styled(
        "─".repeat(area.width as usize),
        theme::text_muted(),
    )))
    .render(
        Rect {
            x: area.x,
            y: area.y.saturating_add(1),
            width: area.width,
            height: 1,
        },
        buf,
    );
}

pub(crate) fn estimate_wrapped_lines(width: u16, prefix_chars: usize, text: &str) -> usize {
    let usable_width = width as usize;
    if usable_width == 0 {
        return 1;
    }
    let first_line_width = usable_width.saturating_sub(prefix_chars).max(1);
    let text_len = text.chars().count();
    if text_len <= first_line_width {
        return 1;
    }
    let remaining = text_len - first_line_width;
    1 + remaining.div_ceil(usable_width.max(1))
}

pub(super) fn dim_backdrop(buf: &mut Buffer, area: Rect) {
    Block::default()
        .style(theme::dim_backdrop())
        .render(area, buf);
}

pub(super) fn panel_block<'a>(title: &'a str, color: Color) -> Block<'a> {
    Block::default().borders(Borders::ALL).title(Span::styled(
        title,
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ))
}

pub(super) fn content_panel_block<'a>(area: Rect, title: &'a str, color: Color) -> Block<'a> {
    let (pad_x, pad_y) = layout::content_panel_padding_for(area);
    content_panel_block_with_padding(title, color, pad_x, pad_y)
}

pub(super) fn content_panel_block_with_padding<'a>(
    title: &'a str,
    color: Color,
    pad_x: u16,
    pad_y: u16,
) -> Block<'a> {
    panel_block(title, color).padding(Padding {
        left: pad_x,
        right: pad_x,
        top: pad_y,
        bottom: pad_y,
    })
}

pub(super) fn modal_frame(buf: &mut Buffer, area: Rect, title: &str, color: Color) -> Rect {
    let block = panel_block(title, color).style(theme::text_primary().bg(theme::SURFACE));
    let inner = block.inner(area);
    block.render(area, buf);
    inner
}

pub(super) fn modal_content_layout(
    area: Rect,
    kind: layout::ModalLayoutKind,
    content_min: u16,
) -> [Rect; 5] {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(layout::modal_content_constraints(
            layout::modal_header_height_for(kind),
            content_min,
        ))
        .split(area);
    [rows[0], rows[1], rows[2], rows[3], rows[4]]
}

pub(super) fn render_status_hint(
    buf: &mut Buffer,
    status_area: Rect,
    hint_area: Rect,
    keys_area: Rect,
    status: &str,
    hint: Option<Line<'static>>,
    keys: &str,
) {
    render_status_line(buf, status_area, status);
    render_hint_line(buf, hint_area, hint);
    render_keys_line(buf, keys_area, keys);
}

pub(super) fn render_status_line(buf: &mut Buffer, area: Rect, status: &str) {
    Paragraph::new(Line::from(status_line_spans(status))).render(area, buf);
}

fn status_line_spans(status: &str) -> Vec<Span<'static>> {
    let mut spans = vec![Span::styled("Status ".to_string(), theme::label_status())];
    let mut segments = status.split(" | ").peekable();
    while let Some(segment) = segments.next() {
        spans.push(Span::styled(
            segment.to_string(),
            Style::default().fg(theme::status_tone(segment)),
        ));
        if segments.peek().is_some() {
            spans.push(Span::styled(" | ".to_string(), theme::text_muted()));
        }
    }
    spans
}

pub(super) fn render_hint_line(buf: &mut Buffer, area: Rect, hint: Option<Line<'static>>) {
    Paragraph::new(Line::from(vec![
        Span::styled("Hint ", theme::label_hint()),
        Span::styled("".to_string(), theme::text_muted()),
    ]))
    .wrap(Wrap { trim: false })
    .render(area, buf);
    if let Some(hint) = hint {
        Paragraph::new(hint).wrap(Wrap { trim: false }).render(
            Rect {
                x: area.x.saturating_add(5),
                y: area.y,
                width: area.width.saturating_sub(5),
                height: area.height,
            },
            buf,
        );
    }
}

pub(super) fn render_keys_line(buf: &mut Buffer, area: Rect, keys: &str) {
    Paragraph::new(Line::from(vec![
        Span::styled("Keys ", theme::label_hint()),
        Span::styled(keys, theme::text_muted()),
    ]))
    .wrap(Wrap { trim: false })
    .render(area, buf);
}

pub(super) fn table_header_row<const N: usize>(labels: [&str; N]) -> Row<'static> {
    Row::new(
        labels
            .into_iter()
            .map(table_header_cell)
            .collect::<Vec<_>>(),
    )
}

pub(super) fn table_header_cell(label: &str) -> Cell<'static> {
    Cell::from(Span::styled(
        label.to_string(),
        theme::text_muted().add_modifier(Modifier::BOLD),
    ))
}

pub(super) fn truncate_end(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_chars {
        return text.to_string();
    }
    if max_chars == 1 {
        return ".".to_string();
    }
    chars[..max_chars - 1].iter().collect::<String>() + "."
}

#[cfg(test)]
mod tests {
    use ratatui::style::Color;
    use ratatui::{buffer::Buffer, layout::Rect};

    use super::{estimate_wrapped_lines, render_tab_bar, status_line_spans};

    #[test]
    fn estimate_wrapped_lines_grows_for_long_summary() {
        let lines = estimate_wrapped_lines(
            40,
            0,
            "Control center for read-only config views, editor handoff, and backend readiness.",
        );
        assert!(lines >= 2, "{lines}");
    }

    #[test]
    fn status_line_colors_each_segment_independently() {
        let spans = status_line_spans("installed 12 | scoop ready | msvc broken");
        let content = spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<Vec<_>>();
        assert_eq!(
            content,
            vec![
                "Status ",
                "installed 12",
                " | ",
                "scoop ready",
                " | ",
                "msvc broken"
            ]
        );
        assert_eq!(spans[1].style.fg, Some(Color::Gray));
        assert_eq!(spans[2].style.fg, Some(Color::DarkGray));
        assert_eq!(spans[3].style.fg, Some(Color::Green));
        assert_eq!(spans[5].style.fg, Some(Color::Red));
    }

    #[test]
    fn tab_bar_renders_without_frame_and_keeps_separator() {
        let area = Rect::new(0, 0, 40, 3);
        let mut buf = Buffer::empty(area);

        render_tab_bar(&mut buf, area, 0);

        let rendered = (0..area.height)
            .map(|y| {
                (0..area.width)
                    .map(|x| buf[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert!(!rendered.contains("Spoon"), "{rendered}");
        assert!(rendered.contains("1 Configure | 2 Tools"), "{rendered}");
        assert!(rendered.contains("────"), "{rendered}");
        assert!(!rendered.contains("┌"), "{rendered}");
        assert!(!rendered.contains("┐"), "{rendered}");
    }
}
