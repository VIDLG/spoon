use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Row, StatefulWidget, Table};

use crate::tui::ToolManagerState;
use crate::tui::{layout, theme};
use crate::view;

use super::super::render_shared::{
    content_panel_block, render_page_shell, table_header_row, truncate_end,
};

pub(super) fn render_tools(
    buf: &mut Buffer,
    state: &mut ToolManagerState,
    area: ratatui::layout::Rect,
    transient_hint: Option<&str>,
) {
    let selected_count = state.selected.iter().filter(|flag| **flag).count();
    let installed = state.statuses.iter().filter(|s| s.is_usable()).count();
    let missing = state.statuses.len().saturating_sub(installed);
    let status = format!(
        "installed {} | missing {} | selected {}",
        installed, missing, selected_count
    );
    let body = render_page_shell(
        buf,
        area,
        1,
        "Unified tools view for status, detail, install, update, and uninstall.",
        &status,
        "Enter detail | Up/Down move | Space toggle | i/u/x action | a/m/p/c select | r refresh latest | ? help | <-/-> page",
        transient_hint,
    );
    let table_layout = layout::tools_table_layout_for(body);
    let tool_w = table_layout.tool_width as usize;
    let tag_w = table_layout.tag_width as usize;
    let status_w = table_layout.status_width as usize;
    let version_w = table_layout.version_width as usize;
    let latest_w = table_layout.latest_width as usize;
    let size_w = table_layout.size_width as usize;

    let table_rows = state.statuses.iter().enumerate().map(|(index, status)| {
        let is_selected = state.table_state.selected() == Some(index);
        let mark = if state.selected[index] { "[x]" } else { "[ ]" };
        let row_view = view::build_tool_status_row(status, &state.statuses);
        let tool_style = if is_selected {
            theme::selected_text(theme::tool_name(status))
        } else {
            theme::tool_name(status)
        };
        let backend_style = if is_selected {
            theme::selected_text(theme::backend_name(status))
        } else {
            theme::backend_name(status)
        };
        Row::new(vec![
            Cell::from(Span::styled(mark.to_string(), theme::text_primary())),
            Cell::from(Span::styled(
                truncate_end(&row_view.display_name, tool_w),
                tool_style,
            )),
            Cell::from(Span::styled(
                truncate_end(&row_view.tag_label, tag_w),
                theme::text_muted(),
            )),
            Cell::from(Span::styled(
                truncate_end(&row_view.status_label, status_w),
                combined_status_style(status),
            )),
            Cell::from(Span::styled(row_view.backend_label.clone(), backend_style)),
            Cell::from(Span::styled(
                truncate_end(&row_view.version, version_w),
                version_style(status),
            )),
            Cell::from(Span::styled(
                truncate_end(&row_view.latest_version, latest_w),
                latest_style(status),
            )),
            Cell::from(Span::styled(
                truncate_end(&row_view.installed_size, size_w),
                size_style(status),
            )),
            Cell::from(action_triplet(
                row_view.install_enabled,
                row_view.update_enabled,
                row_view.uninstall_enabled,
            )),
        ])
    });

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(layout::TOOLS_TABLE_MIN_HEIGHT)])
        .split(body);

    StatefulWidget::render(
        Table::new(table_rows, layout::tools_table_constraints(table_layout))
            .header(table_header_row([
                "SEL", "TOOL", "TAG", "STATUS", "BACKEND", "VER", "LATEST", "SIZE", "ACT",
            ]))
            .column_spacing(table_layout.column_spacing)
            .block(content_panel_block(rows[0], "Tool List", theme::ACCENT_OK))
            .row_highlight_style(theme::selected_row())
            .highlight_symbol("> "),
        rows[0],
        buf,
        &mut state.table_state,
    );
}

fn action_triplet(
    install_enabled: bool,
    update_enabled: bool,
    uninstall_enabled: bool,
) -> Line<'static> {
    Line::from(vec![
        action_span("i", install_enabled, theme::ACCENT_WARN),
        action_span("u", update_enabled, theme::ACCENT_OK),
        action_span("x", uninstall_enabled, theme::ACCENT_DANGER),
    ])
}

fn action_span(label: &'static str, enabled: bool, accent: ratatui::style::Color) -> Span<'static> {
    if enabled {
        Span::styled(
            label,
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled("-", theme::text_muted())
    }
}

fn version_style(status: &crate::status::ToolStatus) -> Style {
    if status.broken {
        Style::default().fg(theme::ACCENT_DANGER)
    } else if status.is_usable() {
        theme::text_primary()
    } else {
        theme::text_muted()
    }
}

fn latest_style(status: &crate::status::ToolStatus) -> Style {
    if status.update_available {
        Style::default()
            .fg(theme::ACCENT_OK)
            .add_modifier(Modifier::BOLD)
    } else if status.latest_version.is_some() {
        theme::text_muted()
    } else {
        theme::text_muted()
    }
}

fn size_style(status: &crate::status::ToolStatus) -> Style {
    if status.installed_size_bytes.is_some() {
        theme::text_muted()
    } else {
        theme::text_muted()
    }
}

fn combined_status_style(status: &crate::status::ToolStatus) -> Style {
    match status.ownership() {
        crate::status::ToolOwnership::Managed => theme::state_ready(),
        crate::status::ToolOwnership::External => {
            if status.broken {
                Style::default().fg(theme::ACCENT_DANGER)
            } else {
                theme::text_muted()
            }
        }
        crate::status::ToolOwnership::Missing => theme::text_muted(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::status::ToolStatus;
    use crate::packages::tool;
    use crate::view;

    use super::action_triplet;

    #[test]
    fn latest_column_only_shows_newer_version_when_update_exists() {
        let jq_tool = tool::all_tools()
            .into_iter()
            .find(|tool| tool.key == "jq")
            .expect("jq tool");
        let status = ToolStatus {
            tool: jq_tool,
            path: Some(PathBuf::from("C:/fake/jq.exe")),
            version: Some("1.8.1".to_string()),
            latest_version: Some("1.8.2".to_string()),
            installed_size_bytes: None,
            update_available: true,
            expected_dir: None,
            available: true,
            broken: false,
        };

        let row = view::build_tool_status_row(&status, &[status.clone()]);
        assert_eq!(row.version, "1.8.1");
        assert_eq!(row.latest_version, "1.8.2");
    }

    #[test]
    fn latest_column_hides_same_version_when_no_update_is_needed() {
        let jq_tool = tool::all_tools()
            .into_iter()
            .find(|tool| tool.key == "jq")
            .expect("jq tool");
        let status = ToolStatus {
            tool: jq_tool,
            path: Some(PathBuf::from("C:/fake/jq.exe")),
            version: Some("1.8.1".to_string()),
            latest_version: Some("1.8.1".to_string()),
            installed_size_bytes: None,
            update_available: false,
            expected_dir: None,
            available: true,
            broken: false,
        };

        let row = view::build_tool_status_row(&status, &[status.clone()]);
        assert_eq!(row.version, "1.8.1");
        assert_eq!(row.latest_version, "-");
    }

    #[test]
    fn tools_table_header_includes_tag_column() {
        let layout =
            crate::tui::layout::tools_table_layout_for(ratatui::layout::Rect::new(0, 0, 140, 40));
        let constraints = crate::tui::layout::tools_table_constraints(layout);
        assert_eq!(constraints.len(), 9);
    }

    #[test]
    fn installed_size_column_formats_human_readable_units() {
        let jq_tool = tool::all_tools()
            .into_iter()
            .find(|tool| tool.key == "jq")
            .expect("jq tool");
        let status = ToolStatus {
            tool: jq_tool,
            path: Some(PathBuf::from("C:/fake/jq.exe")),
            version: Some("1.8.1".to_string()),
            latest_version: Some("1.8.1".to_string()),
            installed_size_bytes: Some(2048),
            update_available: false,
            expected_dir: None,
            available: true,
            broken: false,
        };

        let row = view::build_tool_status_row(&status, &[status.clone()]);
        assert_eq!(row.installed_size, "2.0K");
    }

    #[test]
    fn action_cells_compact_disabled_actions() {
        let enabled: String = action_triplet(true, true, true)
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
        let mixed: String = action_triplet(true, false, true)
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert_eq!(enabled, "iux");
        assert_eq!(mixed, "i-x");
    }
}
