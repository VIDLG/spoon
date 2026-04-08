use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::symbols::line;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget, Wrap,
};

use crate::tui::layout::{self, ModalLayoutKind};
use crate::tui::{HelpState, OutputState, theme};

use super::super::render_shared::{
    compose_hint_line, dim_backdrop, modal_content_layout, modal_frame, render_status_hint,
};

fn output_visual_line_count(viewport: Rect, detail_lines: &[Line<'static>]) -> usize {
    if viewport.width == 0 {
        return 0;
    }

    Paragraph::new(detail_lines.to_vec())
        .wrap(Wrap { trim: true })
        .line_count(viewport.width)
}

fn output_max_scroll(viewport: Rect, detail_lines: &[Line<'static>]) -> u16 {
    let viewport_height = viewport.height as usize;
    if viewport_height == 0 {
        return 0;
    }

    let visual_line_count = output_visual_line_count(viewport, detail_lines);

    visual_line_count.saturating_sub(viewport_height) as u16
}

fn scoop_download_summary(output: &OutputState) -> Option<String> {
    if !output.running
        || !output
            .lines
            .iter()
            .any(|line| line.starts_with("Planned Spoon package action (Scoop): "))
    {
        return None;
    }
    if output
        .lines
        .iter()
        .rev()
        .any(|line| line.to_ascii_lowercase().starts_with("download progress "))
    {
        return None;
    }

    for (index, line) in output.lines.iter().enumerate().rev() {
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("download: [") {
            return Some("aria2 transfer in progress".to_string());
        }
        if lower.starts_with("starting download with aria2") {
            return Some("starting aria2 transfer".to_string());
        }
        if lower.starts_with("downloading http://") || lower.starts_with("downloading https://") {
            return Some(describe_download_line(line));
        }
        if lower == "downloading" {
            if let Some(next_line) = output.lines.get(index + 1) {
                let next_lower = next_line.to_ascii_lowercase();
                if next_lower.starts_with("http://") || next_lower.starts_with("https://") {
                    return Some(describe_download_line(next_line));
                }
            }
            return Some("package archive in progress".to_string());
        }
    }

    None
}

fn describe_download_line(line: &str) -> String {
    if let Some(size_start) = line.rfind('(') {
        if let Some(size_end) = line[size_start + 1..].find(')') {
            let size = line[size_start + 1..size_start + 1 + size_end].trim();
            if !size.is_empty() {
                return format!("package archive in progress ({size})");
            }
        }
    }

    "package archive in progress".to_string()
}

fn scoop_download_indicator_line(summary: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            "Progress ",
            Style::default()
                .fg(theme::ACCENT_WARN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("[{summary}]"),
            Style::default().fg(theme::ACCENT_WARN),
        ),
    ])
}

fn parse_download_progress_line(line: &str) -> Option<(Option<u8>, String)> {
    let rest = line.strip_prefix("Download progress ")?;
    if let Some((percent_text, summary)) = rest.split_once("% ") {
        let percent = percent_text.trim().parse::<u8>().ok()?;
        return Some((Some(percent.min(100)), summary.trim().to_string()));
    }
    Some((None, rest.trim().to_string()))
}

fn render_download_progress_line(line: &str) -> Option<Line<'static>> {
    let (percent, summary) = parse_download_progress_line(line)?;
    let filled = percent.map(|p| ((p as usize) * 20) / 100).unwrap_or(0);
    let empty = 20usize.saturating_sub(filled);
    let bar = format!("{}{}", "=".repeat(filled), " ".repeat(empty));

    let mut spans = vec![
        Span::styled(
            "Progress ",
            Style::default()
                .fg(theme::ACCENT_WARN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("[", Style::default().fg(theme::ACCENT_WARN)),
        Span::styled(bar, Style::default().fg(theme::ACCENT_OK)),
        Span::styled("]", Style::default().fg(theme::ACCENT_WARN)),
    ];
    if let Some(percent) = percent {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            format!("{percent:>3}%"),
            Style::default()
                .fg(theme::ACCENT_OK)
                .add_modifier(Modifier::BOLD),
        ));
    }
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        summary,
        Style::default().fg(theme::ACCENT_WARN),
    ));

    Some(Line::from(spans))
}

pub(super) fn render_output_modal(
    buf: &mut Buffer,
    area: Rect,
    output: &mut OutputState,
    transient_hint: Option<&str>,
) {
    let popup = layout::modal_rect(area, layout::modal_size_for(ModalLayoutKind::Output));
    let status = output.status.as_str();
    let hint = compose_hint_line(transient_hint);
    let keys = if output.running {
        "Esc/q cancel | c copy full log | Up/Down/Page scroll | Home/End"
    } else {
        "Enter/Esc close | c copy full log | q quit | Up/Down/Page scroll | Home/End"
    };
    let frame_title = format!("Output / {}", output.title);
    let inner = {
        dim_backdrop(buf, area);
        Clear.render(popup, buf);
        modal_frame(buf, popup, &frame_title, theme::ACCENT_WARN)
    };
    let [_header, content, status_row, hint_row, keys_row] = modal_content_layout(
        inner,
        ModalLayoutKind::Output,
        layout::modal_content_min_height_for(popup, ModalLayoutKind::Output),
    );
    let detail_lines = output
        .lines
        .iter()
        .map(|line| output_line(line))
        .collect::<Vec<_>>();
    let mut detail_lines = detail_lines;
    if let Some(summary) = scoop_download_summary(output) {
        detail_lines.push(scoop_download_indicator_line(&summary));
    }
    let viewport = layout::modal_body_rect_with_title_gap(content);
    let max_scroll = output_max_scroll(viewport, &detail_lines);
    output.max_scroll = max_scroll;
    output.page_step = viewport.height.max(1);
    if output.snap_to_bottom_on_render {
        output.scroll = max_scroll;
        output.snap_to_bottom_on_render = false;
    }
    let scroll = if output.auto_scroll {
        max_scroll
    } else {
        output.scroll.min(max_scroll)
    };
    Paragraph::new(detail_lines)
        .style(theme::text_primary())
        .wrap(Wrap { trim: true })
        .scroll((scroll, 0))
        .render(viewport, buf);
    if max_scroll > 0 {
        let mut scrollbar_state = ScrollbarState::new(max_scroll as usize + 1)
            .position(scroll as usize)
            .viewport_content_length(viewport.height as usize);
        Scrollbar::new(ScrollbarOrientation::VerticalRight).render(
            viewport,
            buf,
            &mut scrollbar_state,
        );
    }
    render_separator(buf, status_row);
    render_status_hint(buf, status_row, hint_row, keys_row, status, hint, keys);
}

fn render_separator(buf: &mut Buffer, area: Rect) {
    if area.width == 0 || area.y == 0 {
        return;
    }
    let y = area.y - 1;
    let style = theme::text_muted();
    for x in area.x..area.x + area.width {
        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_symbol(line::NORMAL.horizontal).set_style(style);
        }
    }
}

pub(super) fn render_help_modal(buf: &mut Buffer, area: Rect, help: &HelpState) {
    let popup = layout::modal_rect(area, layout::modal_size_for(ModalLayoutKind::Help));
    dim_backdrop(buf, area);
    Clear.render(popup, buf);
    let inner = modal_frame(buf, popup, &help.title, theme::ACCENT_WARN);
    let body = layout::modal_body_rect_with_title_gap(inner);
    Paragraph::new(
        help.lines
            .iter()
            .cloned()
            .map(Line::from)
            .collect::<Vec<_>>(),
    )
    .wrap(Wrap { trim: false })
    .scroll((help.scroll, 0))
    .style(theme::text_primary().bg(theme::SURFACE))
    .render(body, buf);
}

fn output_line(line: &str) -> Line<'static> {
    if let Some(progress_line) = render_download_progress_line(line) {
        return progress_line;
    }

    let lower = line.to_ascii_lowercase();
    let style = if lower.starts_with("== ") {
        theme::text_primary().add_modifier(Modifier::BOLD)
    } else if lower.starts_with("> ") || line.starts_with("Planned Spoon package action (Scoop): ")
    {
        theme::field_backend()
    } else if lower.contains("requires a configured root")
        || lower.contains("configure a root before")
        || lower.contains("configure a root before")
        || lower.contains("set root")
        || lower.contains("install or repair the scoop backend")
        || lower.contains("before installing scoop")
        || lower.contains("before managing the toolchain")
    {
        Style::default()
            .fg(theme::ACCENT_WARN)
            .add_modifier(Modifier::BOLD)
    } else if lower.starts_with("error:") {
        Style::default()
            .fg(theme::ACCENT_DANGER)
            .add_modifier(Modifier::BOLD)
    } else if lower.starts_with("warning:") {
        Style::default().fg(theme::ACCENT_WARN)
    } else {
        theme::text_primary()
    };

    Line::from(Span::styled(line.to_string(), style))
}

#[cfg(test)]
mod tests {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::text::Line;

    use crate::tui::OutputState;

    use super::{
        output_max_scroll, output_visual_line_count, parse_download_progress_line,
        render_download_progress_line, render_separator, scoop_download_summary,
    };

    #[test]
    fn wrapped_output_lines_increase_scroll_range() {
        let viewport = Rect::new(0, 0, 28, 6);
        let detail_lines = vec![
            Line::from("short"),
            Line::from(
                "this is a very long output line that should wrap across multiple visual rows",
            ),
            Line::from(
                "another very long output line that should also wrap across multiple visual rows",
            ),
        ];

        let max_scroll = output_max_scroll(viewport, &detail_lines);
        assert!(max_scroll > 0);
    }

    #[test]
    fn wrapped_output_lines_count_visual_rows() {
        let viewport = Rect::new(0, 0, 18, 6);
        let detail_lines = vec![
            Line::from("12345678901234567890"),
            Line::from("123456789012345678901234567890"),
        ];

        let visual_count = output_visual_line_count(viewport, &detail_lines);
        assert!(visual_count >= 4);
    }

    #[test]
    fn scoop_download_activity_detection_requires_running_scoop_download() {
        let mut output = OutputState {
            title: "install glow".to_string(),
            status: "running command".to_string(),
            lines: vec![
                "Planned Spoon package action (Scoop): install glow --no-update-scoop".to_string(),
                "Downloading https://example.invalid/glow.zip (6.2 MB)...".to_string(),
            ],
            scroll: 0,
            max_scroll: 0,
            page_step: 10,
            auto_scroll: true,
            snap_to_bottom_on_render: true,
            running: true,
            follow_up: None,
        };
        assert_eq!(
            scoop_download_summary(&output),
            Some("package archive in progress (6.2 MB)".to_string())
        );

        output.running = false;
        assert_eq!(scoop_download_summary(&output), None);
    }

    #[test]
    fn scoop_download_activity_detection_supports_split_downloading_lines() {
        let output = OutputState {
            title: "install claude-code".to_string(),
            status: "running command".to_string(),
            lines: vec![
                "Planned Spoon package action (Scoop): install claude-code --no-update-scoop"
                    .to_string(),
                "Installing 'claude-code' (2.1.74) [64bit] from 'main' bucket".to_string(),
                "Downloading".to_string(),
                "https://storage.googleapis.com/example/claude.exe (227.8 MB)...".to_string(),
            ],
            scroll: 0,
            max_scroll: 0,
            page_step: 10,
            auto_scroll: true,
            snap_to_bottom_on_render: true,
            running: true,
            follow_up: None,
        };

        assert_eq!(
            scoop_download_summary(&output),
            Some("package archive in progress (227.8 MB)".to_string())
        );
    }

    #[test]
    fn scoop_download_summary_yields_to_real_progress_lines() {
        let output = OutputState {
            title: "install claude-code".to_string(),
            status: "running command".to_string(),
            lines: vec![
                "Planned Spoon package action (Scoop): install claude-code".to_string(),
                "Downloading".to_string(),
                "https://example.invalid/claude.exe (227.8 MB)...".to_string(),
                "Download progress 34% (77.1 MB / 227.8 MB)".to_string(),
            ],
            scroll: 0,
            max_scroll: 0,
            page_step: 10,
            auto_scroll: true,
            snap_to_bottom_on_render: true,
            running: true,
            follow_up: None,
        };

        assert_eq!(scoop_download_summary(&output), None);
    }

    #[test]
    fn parse_download_progress_line_extracts_percent_and_summary() {
        assert_eq!(
            parse_download_progress_line("Download progress 34% (77.1 MB / 227.8 MB)"),
            Some((Some(34), "(77.1 MB / 227.8 MB)".to_string()))
        );
    }

    #[test]
    fn render_download_progress_line_builds_visual_bar() {
        let rendered = render_download_progress_line("Download progress 34% (77.1 MB / 227.8 MB)")
            .expect("progress line");
        let text = rendered
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();
        assert!(text.contains("Progress ["));
        assert!(text.contains(" 34%"));
        assert!(text.contains("(77.1 MB / 227.8 MB)"));
    }

    #[test]
    fn parse_download_progress_line_accepts_activity_without_total() {
        assert_eq!(
            parse_download_progress_line(
                "Download progress (12.3 MB downloaded) Installers\\sdk-tools.msi"
            ),
            Some((
                None,
                "(12.3 MB downloaded) Installers\\sdk-tools.msi".to_string()
            ))
        );
    }

    #[test]
    fn render_download_progress_line_without_total_still_shows_activity() {
        let rendered = render_download_progress_line(
            "Download progress (12.3 MB downloaded) Installers\\sdk-tools.msi",
        )
        .expect("progress line");
        let text = rendered
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();
        assert!(text.contains("Progress ["));
        assert!(text.contains("(12.3 MB downloaded)"));
        assert!(!text.contains('%'));
    }

    #[test]
    fn render_separator_draws_rule_above_status_row() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 4));
        render_separator(&mut buf, Rect::new(0, 2, 10, 1));
        let row = (0..10).map(|x| buf[(x, 1)].symbol()).collect::<String>();
        assert_eq!(row, "──────────");
    }
}
