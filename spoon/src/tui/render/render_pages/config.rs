use ratatui::buffer::Buffer;
use ratatui::layout::{Direction, Layout};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, StatefulWidget};

use crate::packages::{self, ConfigBadgeTone};
use crate::status::ToolStatus;
use crate::tui::{AppConfigSnapshot, ConfigMenuAction, config_menu_items, layout, theme};
use crate::view;

use super::super::render_shared::{content_panel_block, render_page_shell};

pub(super) fn render_config_menu(
    buf: &mut Buffer,
    state: &mut ListState,
    area: ratatui::layout::Rect,
    transient_hint: Option<&str>,
    _statuses_snapshot: &[ToolStatus],
    config_snapshot: &AppConfigSnapshot,
) {
    let config_view = view::build_config_model();

    let body = render_page_shell(
        buf,
        area,
        0,
        "Control center for read-only config views and editor handoff.",
        &format!(
            "root {} | editor {}",
            display_or_unset(&config_view.root_path),
            if config_snapshot.editor_available {
                "ready"
            } else {
                "missing"
            }
        ),
        "Enter open | Up/Down move | r refresh latest | ? help | <-/-> page | q quit",
        transient_hint,
    );

    let items: Vec<ListItem> = config_menu_items()
        .into_iter()
        .map(|(label, action)| {
            let (tag, tag_style) = match action {
                ConfigMenuAction::Editor => {
                    if config_snapshot.editor_available {
                        ("ready", theme::state_ready())
                    } else {
                        ("missing", theme::state_missing())
                    }
                }
                ConfigMenuAction::Global => {
                    if config_view.root_path.trim().is_empty() {
                        ("needs root", theme::state_missing())
                    } else {
                        ("ready", theme::state_ready())
                    }
                }
                ConfigMenuAction::Package(package_key) => config_tag_for_package(package_key),
            };
            let summary_lines = config_item_summary_lines(action, &config_view, config_snapshot);
            let mut lines = vec![Line::from(vec![
                Span::styled(label.to_string(), theme::text_primary()),
                Span::styled(" [".to_string(), theme::text_muted()),
                Span::styled(tag.to_string(), tag_style),
                Span::styled("]".to_string(), theme::text_muted()),
            ])];
            lines.extend(summary_lines.into_iter().map(|summary| {
                Line::from(Span::styled(format!("  {summary}"), theme::text_muted()))
            }));
            ListItem::new(lines)
        })
        .collect();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(layout::config_page_constraints(body))
        .split(body);

    let list = List::new(items)
        .block(content_panel_block(
            rows[2],
            "Configure",
            theme::ACCENT_WARN,
        ))
        .highlight_style(theme::selected_row())
        .highlight_symbol("> ");
    StatefulWidget::render(list, rows[2], buf, state);
}

fn config_item_summary_lines(
    action: ConfigMenuAction,
    config_view: &view::ConfigModel,
    config_snapshot: &AppConfigSnapshot,
) -> Vec<String> {
    match action {
        ConfigMenuAction::Editor => vec![format!(
            "command: {} | status: {}",
            config_snapshot.editor_command,
            if config_snapshot.editor_available {
                "ready"
            } else {
                "missing"
            }
        )],
        ConfigMenuAction::Global => vec![
            format!(
                "root: {} | proxy: {}",
                display_or_unset(&config_view.root_path),
                display_option(config_view.runtime_proxy.as_deref())
            ),
            format!(
                "editor: {}",
                display_option(config_view.runtime_editor.as_deref())
            ),
        ],
        ConfigMenuAction::Package(package_key) => package_summary_lines(package_key, config_view),
    }
}

fn package_summary_lines(package_key: &str, config_view: &view::ConfigModel) -> Vec<String> {
    let mut lines = packages::config_menu_summary_lines(package_key);
    if lines.is_empty()
        && let Some(package) = config_view
            .packages
            .iter()
            .find(|package| package.key == package_key)
    {
        let summary = package
            .entries
            .iter()
            .map(|entry| format!("{}: {}", entry.key, entry.value.display_value()))
            .collect::<Vec<_>>()
            .join(" | ");
        if !summary.is_empty() {
            lines.push(summary);
        }
    }
    if let Some(scope) = view::build_package_config_scope_model(package_key)
        && !scope.conflicts.is_empty()
    {
        lines.push("drift detected".to_string());
    }
    lines
}

fn config_tag_for_package(package_key: &str) -> (&'static str, ratatui::style::Style) {
    match packages::config_target_badge(package_key) {
        Some(badge) => (
            badge.label,
            match badge.tone {
                ConfigBadgeTone::Ready => theme::state_ready(),
                ConfigBadgeTone::Missing => theme::state_missing(),
                ConfigBadgeTone::Muted => theme::text_muted(),
            },
        ),
        None => ("missing", theme::state_missing()),
    }
}

fn display_or_unset(value: &str) -> String {
    if value.trim().is_empty() {
        "unset".to_string()
    } else {
        value.to_string()
    }
}

fn display_option(value: Option<&str>) -> String {
    value
        .filter(|item| !item.trim().is_empty())
        .unwrap_or("unset")
        .to_string()
}
