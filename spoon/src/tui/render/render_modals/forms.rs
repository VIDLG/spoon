use ratatui::buffer::Buffer;
use ratatui::layout::{Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, List, ListItem, Paragraph, Widget, Wrap};

use crate::editor;
use crate::packages;
use crate::tui::layout::{self, ModalLayoutKind};
use crate::tui::{ConfigKind, EditorSetupState, FormState, theme};
use crate::view::{self, ConfigDetailSection};

use super::super::render_shared::{
    compose_hint_line, content_panel_block_with_padding, dim_backdrop, estimate_wrapped_lines,
    modal_frame, render_status_hint,
};

pub(super) fn render_form_modal(
    buf: &mut Buffer,
    area: Rect,
    form: &FormState,
    transient_hint: Option<&str>,
) {
    let subtitle = match form.kind {
        ConfigKind::Global => {
            "Inspect the current Spoon-owned configuration, then open the real config file when you want to edit it."
        }
        ConfigKind::Package(_) => {
            "Inspect the current package-specific configuration, then open the real config location when you want to edit it."
        }
    };
    let mode_label = "Mode: Current configuration";
    let editable = is_editable_target(form.kind);
    let status = if editable {
        "read-only view | open native config to edit".to_string()
    } else {
        "read-only view | no native config file to edit".to_string()
    };
    let keys = if editable {
        "Enter/e open editor | o open folder | Esc close"
    } else {
        "Esc close"
    };
    let hint = compose_hint_line(transient_hint);
    let popup = layout::modal_rect(area, layout::modal_size_for(ModalLayoutKind::Form));
    dim_backdrop(buf, area);
    Clear.render(popup, buf);
    let inner = modal_frame(buf, popup, "Configuration", theme::ACCENT_WARN);
    let body = layout::modal_body_rect(inner);
    let subtitle_height = estimate_wrapped_lines(body.width, 0, subtitle).clamp(1, 3) as u16;
    let header_height = 2 + subtitle_height;
    let panel_gap = 1;
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(header_height),
            ratatui::layout::Constraint::Length(panel_gap),
            ratatui::layout::Constraint::Min(layout::modal_content_min_height_for(
                popup,
                ModalLayoutKind::Form,
            )),
            ratatui::layout::Constraint::Length(layout::MODAL_STATUS_HEIGHT),
            ratatui::layout::Constraint::Length(layout::MODAL_HINT_HEIGHT),
            ratatui::layout::Constraint::Length(layout::MODAL_KEYS_HEIGHT),
        ])
        .split(body);
    let [header, _gap, content, status_row, hint_row, keys_row] =
        [rows[0], rows[1], rows[2], rows[3], rows[4], rows[5]];
    Paragraph::new(vec![
        Line::from(Span::styled(
            form.title,
            Style::default()
                .fg(theme::ACCENT_WARN)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(mode_label, theme::text_muted())),
        Line::from(Span::styled(subtitle, theme::text_primary())),
    ])
    .wrap(Wrap { trim: false })
    .render(header, buf);
    let detail_lines = form_detail_lines(form);
    Paragraph::new(detail_lines)
        .block(content_panel_block_with_padding(
            "Current configuration",
            theme::ACCENT_WARN,
            1,
            0,
        ))
        .wrap(Wrap { trim: false })
        .render(content, buf);
    render_status_hint(buf, status_row, hint_row, keys_row, &status, hint, keys);
}

pub(super) fn render_editor_setup_modal(
    buf: &mut Buffer,
    area: Rect,
    setup: &EditorSetupState,
    transient_hint: Option<&str>,
) {
    let popup = layout::modal_rect(area, layout::modal_size_for(ModalLayoutKind::EditorSetup));
    dim_backdrop(buf, area);
    Clear.render(popup, buf);
    let inner = modal_frame(buf, popup, "Editor Setup", theme::ACCENT_WARN);
    let body = layout::modal_body_rect(inner);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(layout::modal_header_height_for(
                ModalLayoutKind::EditorSetup,
            )),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Min(layout::modal_content_min_height_for(
                popup,
                ModalLayoutKind::EditorSetup,
            )),
            ratatui::layout::Constraint::Length(layout::MODAL_STATUS_HEIGHT),
            ratatui::layout::Constraint::Length(layout::MODAL_HINT_HEIGHT),
            ratatui::layout::Constraint::Length(layout::MODAL_KEYS_HEIGHT),
        ])
        .split(body);
    let [header, _gap, content, status_row, hint_row, keys_row] =
        [rows[0], rows[1], rows[2], rows[3], rows[4], rows[5]];
    let selected = editor::candidates()[setup.selected];
    let selected_available = editor::is_candidate_available(selected);
    let selected_managed = editor::is_candidate_managed(selected);
    let selected_external = editor::is_candidate_external(selected);
    let selected_default = editor::is_default_candidate(selected);
    let status = if selected_default && selected_managed && selected_available {
        "selected default | managed"
    } else if selected_default && selected_external {
        "selected default | external"
    } else if selected_default {
        "selected default | missing"
    } else if selected_managed && selected_available {
        "selected managed"
    } else if selected_external {
        "selected external"
    } else {
        "selected missing"
    };
    let hint = compose_hint_line(setup.notice.as_deref().or(transient_hint));
    let keys = if selected_managed {
        "Enter set default | u uninstall | x clear default | Up/Down move | Esc close"
    } else if selected_available {
        "Enter set default | x clear default | Up/Down move | Esc close"
    } else {
        "Enter install | u uninstall | x clear default | Up/Down move | Esc close"
    };
    Paragraph::new(vec![
        Line::from(Span::styled(
            "Configured editor is not available.",
            Style::default()
                .fg(theme::ACCENT_WARN)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            match setup.requested_kind {
                Some(crate::tui::ConfigKind::Global) => {
                    "Requested target: Global settings".to_string()
                }
                Some(crate::tui::ConfigKind::Package(package_key)) => {
                    let title = packages::config_target_descriptors()
                        .into_iter()
                        .find(|descriptor| descriptor.package_key == package_key)
                        .map(|descriptor| descriptor.detail_title)
                        .unwrap_or(package_key);
                    format!("Requested target: {title} config")
                }
                None => "Requested target: editor setup".to_string(),
            },
            theme::text_primary(),
        )),
        Line::from(Span::styled(
            format!("Current command: {}", setup.current_command),
            theme::field_backend(),
        )),
        Line::from(Span::styled(
            "Default means the editor spoon uses to open config files.",
            theme::text_primary(),
        )),
        Line::from(Span::styled(
            "Install, set default, uninstall, or clear the default command here.",
            theme::text_primary(),
        )),
    ])
    .render(header, buf);

    let items = editor::candidates()
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            let prefix = if setup.selected == index { "> " } else { "  " };
            let mut spans = vec![
                Span::styled(prefix, theme::text_primary()),
                Span::styled(candidate.label, theme::text_primary()),
                Span::styled(format!(" ({})", candidate.command), theme::field_backend()),
            ];
            if index == editor::recommended_candidate_index() {
                spans.push(Span::styled(" [".to_string(), theme::text_muted()));
                spans.push(Span::styled(
                    "recommended".to_string(),
                    theme::state_recommended(),
                ));
                if editor::is_default_candidate(*candidate)
                    || editor::is_candidate_available(*candidate)
                {
                    spans.push(Span::styled(", ".to_string(), theme::text_muted()));
                } else {
                    spans.push(Span::styled("]".to_string(), theme::text_muted()));
                }
            }
            if editor::is_candidate_managed(*candidate)
                && editor::is_candidate_available(*candidate)
            {
                if index != editor::recommended_candidate_index() {
                    spans.push(Span::styled(" [".to_string(), theme::text_muted()));
                }
                spans.push(Span::styled("managed".to_string(), theme::state_ready()));
                if editor::is_default_candidate(*candidate) {
                    spans.push(Span::styled(", ".to_string(), theme::text_muted()));
                } else {
                    spans.push(Span::styled("]".to_string(), theme::text_muted()));
                }
            } else if editor::is_candidate_external(*candidate) {
                if index != editor::recommended_candidate_index() {
                    spans.push(Span::styled(" [".to_string(), theme::text_muted()));
                }
                spans.push(Span::styled("external".to_string(), theme::text_muted()));
                if editor::is_default_candidate(*candidate) {
                    spans.push(Span::styled(", ".to_string(), theme::text_muted()));
                } else {
                    spans.push(Span::styled("]".to_string(), theme::text_muted()));
                }
            }
            if editor::is_default_candidate(*candidate) {
                if index != editor::recommended_candidate_index()
                    && !editor::is_candidate_available(*candidate)
                {
                    spans.push(Span::styled(" [".to_string(), theme::text_muted()));
                }
                spans.push(Span::styled("default".to_string(), theme::state_default()));
                spans.push(Span::styled("]".to_string(), theme::text_muted()));
            }
            ListItem::new(Line::from(spans))
        })
        .collect::<Vec<_>>();
    Widget::render(
        List::new(items).block(content_panel_block_with_padding(
            "Editors",
            theme::ACCENT_WARN,
            1,
            0,
        )),
        content,
        buf,
    );
    render_status_hint(buf, status_row, hint_row, keys_row, status, hint, keys);
}

fn form_detail_lines(form: &FormState) -> Vec<Line<'static>> {
    match form.kind {
        ConfigKind::Global => global_detail_lines(),
        ConfigKind::Package(package_key) => package_detail_lines(package_key),
    }
}

fn global_detail_lines() -> Vec<Line<'static>> {
    let model = view::build_config_model();
    vec![
        section_line("Files"),
        detail_line(&format!("config.toml: {}", model.config_file)),
        blank_line(),
        section_line("Runtime"),
        detail_line(&format!("root: {}", display_or_unset(&model.root_path))),
        detail_line(&format!(
            "proxy: {}",
            display_option(model.runtime_proxy.as_deref()),
        )),
        detail_line(&format!(
            "editor: {}",
            display_option(model.runtime_editor.as_deref())
        )),
        detail_line(&format!("msvc_arch: {}", model.runtime_msvc_arch)),
        blank_line(),
        section_line("Derived"),
        detail_line(&format!("scoop_root: {}", model.derived_scoop_root)),
        detail_line(&format!(
            "managed_msvc_root: {}",
            model.derived_managed_msvc_root
        )),
        detail_line(&format!(
            "managed_msvc_toolchain: {}",
            model.derived_managed_msvc_toolchain
        )),
        detail_line(&format!(
            "official_msvc_root: {}",
            model.derived_official_msvc_root
        )),
    ]
}

fn package_detail_lines(package_key: &str) -> Vec<Line<'static>> {
    let sections = view::build_package_config_detail_sections(package_key);
    if sections.is_empty() {
        vec![detail_line("No package configuration view is registered.")]
    } else {
        config_detail_section_lines(&sections)
    }
}

fn config_detail_section_lines(sections: &[ConfigDetailSection]) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for section in sections {
        if !lines.is_empty() {
            lines.push(blank_line());
        }
        lines.push(section_line(&section.title));
        for entry in &section.entries {
            lines.push(detail_line(&format!(
                "{}: {}",
                entry.key,
                entry.value.display_value()
            )));
        }
    }
    lines
}

fn detail_line(text: &str) -> Line<'static> {
    Line::from(Span::styled(text.to_string(), theme::text_primary()))
}

fn section_line(text: &str) -> Line<'static> {
    Line::from(Span::styled(
        text.to_string(),
        theme::text_muted().add_modifier(Modifier::BOLD),
    ))
}

fn blank_line() -> Line<'static> {
    Line::from("")
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

fn is_editable_target(kind: ConfigKind) -> bool {
    match kind {
        ConfigKind::Global => true,
        ConfigKind::Package(package_key) => packages::config_target_descriptors()
            .into_iter()
            .find(|descriptor| descriptor.package_key == package_key)
            .map(|descriptor| descriptor.editable)
            .unwrap_or(false),
    }
}

#[cfg(test)]
mod tests {
    use super::{display_option, display_or_unset};

    #[test]
    fn display_helpers_show_unset_for_empty_values() {
        assert_eq!(display_or_unset(""), "unset");
        assert_eq!(display_option(None), "unset");
    }
}
