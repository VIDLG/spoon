use ratatui::style::{Color, Modifier, Style};

use crate::status::ToolStatus;

#[cfg(test)]
use crate::status::ManagedReadiness;

pub(crate) const SURFACE: Color = Color::Black;
pub(crate) const BACKDROP: Color = Color::Black;

pub(crate) const TEXT_PRIMARY: Color = Color::Gray;
pub(crate) const TEXT_MUTED: Color = Color::DarkGray;
pub(crate) const TEXT_TOOL_READY: Color = Color::White;
pub(crate) const ACCENT_BRAND: Color = Color::Cyan;
pub(crate) const ACCENT_INFO: Color = Color::Blue;
pub(crate) const ACCENT_WARN: Color = Color::Yellow;
pub(crate) const ACCENT_OK: Color = Color::Green;
pub(crate) const ACCENT_DANGER: Color = Color::Red;
pub(crate) const FIELD_PACKAGE: Color = Color::Cyan;
pub(crate) const FIELD_BACKEND: Color = Color::Gray;
pub(crate) const FIELD_PATH: Color = Color::DarkGray;
pub(crate) const FIELD_VERSION: Color = Color::Gray;
pub(crate) const SELECTION_BG: Color = Color::Rgb(24, 49, 83);

pub(crate) fn tabs_base() -> Style {
    Style::default().fg(TEXT_MUTED)
}

pub(crate) fn tabs_selected() -> Style {
    Style::default()
        .fg(ACCENT_BRAND)
        .add_modifier(Modifier::BOLD)
}

pub(crate) fn label_status() -> Style {
    Style::default().fg(TEXT_MUTED).add_modifier(Modifier::BOLD)
}

pub(crate) fn label_hint() -> Style {
    Style::default().fg(TEXT_MUTED).add_modifier(Modifier::BOLD)
}

pub(crate) fn text_primary() -> Style {
    Style::default().fg(TEXT_PRIMARY)
}

pub(crate) fn text_muted() -> Style {
    Style::default().fg(TEXT_MUTED)
}

pub(crate) fn field_package() -> Style {
    Style::default()
        .fg(FIELD_PACKAGE)
        .add_modifier(Modifier::BOLD)
}

pub(crate) fn field_backend() -> Style {
    Style::default()
        .fg(FIELD_BACKEND)
        .add_modifier(Modifier::BOLD)
}

pub(crate) fn backend_name(status: &ToolStatus) -> Style {
    if status.broken {
        Style::default()
            .fg(ACCENT_DANGER)
            .add_modifier(Modifier::BOLD)
    } else if status.is_usable() {
        field_backend()
    } else if status.is_detected() {
        Style::default()
            .fg(ACCENT_WARN)
            .add_modifier(Modifier::BOLD)
    } else {
        text_muted().add_modifier(Modifier::BOLD)
    }
}

pub(crate) fn field_path() -> Style {
    Style::default().fg(FIELD_PATH)
}

pub(crate) fn field_version() -> Style {
    Style::default().fg(FIELD_VERSION)
}

pub(crate) fn state_ready() -> Style {
    Style::default().fg(ACCENT_OK).add_modifier(Modifier::BOLD)
}

pub(crate) fn state_missing() -> Style {
    Style::default()
        .fg(ACCENT_WARN)
        .add_modifier(Modifier::BOLD)
}

#[cfg(test)]
pub(crate) fn state_broken() -> Style {
    Style::default()
        .fg(ACCENT_DANGER)
        .add_modifier(Modifier::BOLD)
}

pub(crate) fn state_recommended() -> Style {
    Style::default()
        .fg(ACCENT_INFO)
        .add_modifier(Modifier::BOLD)
}

pub(crate) fn state_default() -> Style {
    Style::default()
        .fg(ACCENT_BRAND)
        .add_modifier(Modifier::BOLD)
}

#[cfg(test)]
pub(crate) fn readiness_style(readiness: ManagedReadiness) -> Style {
    match readiness {
        ManagedReadiness::Ready => state_ready(),
        ManagedReadiness::Broken => state_broken(),
        ManagedReadiness::Missing | ManagedReadiness::Detected => state_missing(),
    }
}

pub(crate) fn tool_name(status: &ToolStatus) -> Style {
    if status.broken {
        Style::default()
            .fg(ACCENT_DANGER)
            .add_modifier(Modifier::BOLD)
    } else if status.is_usable() {
        Style::default()
            .fg(TEXT_TOOL_READY)
            .add_modifier(Modifier::BOLD)
    } else if status.is_detected() {
        Style::default()
            .fg(ACCENT_WARN)
            .add_modifier(Modifier::BOLD)
    } else {
        text_muted().add_modifier(Modifier::BOLD)
    }
}

pub(crate) fn selected_row() -> Style {
    Style::default()
        .bg(SELECTION_BG)
        .add_modifier(Modifier::BOLD)
}

pub(crate) fn selected_text(base: Style) -> Style {
    base.bg(SELECTION_BG).add_modifier(Modifier::BOLD)
}

pub(crate) fn dim_backdrop() -> Style {
    Style::default().bg(BACKDROP).add_modifier(Modifier::DIM)
}

pub(crate) fn status_tone(status: &str) -> Color {
    let lower = status.to_ascii_lowercase();
    if lower.contains("running") || lower.contains("starting") {
        TEXT_PRIMARY
    } else if lower.contains("completed")
        || lower.contains("started")
        || lower.contains("success")
        || lower.contains("ready")
        || lower.contains("configured")
    {
        ACCENT_OK
    } else if lower.contains("broken") || lower.contains("error") || lower.contains("failed") {
        ACCENT_DANGER
    } else if lower.contains("blocked")
        || lower.contains("prerequisite")
        || lower.contains("requires")
        || lower.contains("missing")
        || lower.contains("selected")
        || lower.contains("detected")
    {
        ACCENT_WARN
    } else {
        TEXT_PRIMARY
    }
}

pub(crate) fn hint_tone(hint: &str) -> Color {
    let lower = hint.to_ascii_lowercase();
    if lower.contains("failed") || lower.contains("error") || lower.contains("broken") {
        ACCENT_DANGER
    } else if lower.contains("blocked")
        || lower.contains("cannot")
        || lower.contains("not available")
        || lower.contains("missing")
        || lower.contains("requires")
        || lower.contains("wait")
    {
        ACCENT_WARN
    } else {
        TEXT_PRIMARY
    }
}

#[cfg(test)]
mod tests {
    use ratatui::style::Color;

    use crate::status::ManagedReadiness;

    use super::{ACCENT_DANGER, readiness_style};

    #[test]
    fn broken_readiness_uses_danger_red() {
        assert_eq!(
            readiness_style(ManagedReadiness::Broken).fg,
            Some(Color::Red)
        );
        assert_eq!(
            readiness_style(ManagedReadiness::Broken).fg,
            Some(ACCENT_DANGER)
        );
    }
}
