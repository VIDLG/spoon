use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub(crate) const TAB_BAR_HEIGHT: u16 = 2;
pub(crate) const PAGE_SUMMARY_MIN_HEIGHT: u16 = 1;
pub(crate) const PAGE_SUMMARY_MAX_HEIGHT: u16 = 3;
pub(crate) const STATUS_BAR_HEIGHT: u16 = 1;
pub(crate) const HINT_BAR_MIN_HEIGHT: u16 = 1;
pub(crate) const HINT_BAR_MAX_HEIGHT: u16 = 3;
pub(crate) const KEYS_BAR_HEIGHT: u16 = 1;
pub(crate) const PAGE_CONTENT_MIN_HEIGHT: u16 = 8;
pub(crate) const MODAL_STATUS_HEIGHT: u16 = 1;
pub(crate) const MODAL_HINT_HEIGHT: u16 = 1;
pub(crate) const MODAL_KEYS_HEIGHT: u16 = 1;
pub(crate) const MODAL_BODY_PADDING_X: u16 = 1;
pub(crate) const MODAL_BODY_PADDING_Y: u16 = 0;

pub(crate) const CONFIG_TARGETS_MIN_HEIGHT: u16 = 8;
pub(crate) const TOOLS_TABLE_MIN_HEIGHT: u16 = 8;
pub(crate) const CONTENT_PANEL_PADDING_X: u16 = 1;
pub(crate) const CONTENT_PANEL_PADDING_Y: u16 = 0;

pub(crate) const OUTPUT_CONTENT_MIN_HEIGHT: u16 = 3;
pub(crate) const FORM_CONTENT_MIN_HEIGHT: u16 = 6;
pub(crate) const EDITOR_SETUP_CONTENT_MIN_HEIGHT: u16 = 7;

#[derive(Clone, Copy)]
pub(crate) enum ModalSize {
    Compact,
    Standard,
    Wide,
}

impl ModalSize {
    fn percents(self) -> (u16, u16) {
        match self {
            ModalSize::Compact => (48, 24),
            ModalSize::Standard => (76, 76),
            ModalSize::Wide => (90, 86),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum ModalLayoutKind {
    ToolDetail,
    Form,
    EditorSetup,
    Output,
    DebugLog,
    Help,
    CancelRunningConfirm,
    QuitConfirm,
}

#[derive(Clone, Copy)]
pub(crate) struct ToolsTableLayout {
    pub(crate) tool_width: u16,
    pub(crate) tag_width: u16,
    pub(crate) status_width: u16,
    pub(crate) version_width: u16,
    pub(crate) latest_width: u16,
    pub(crate) size_width: u16,
    pub(crate) column_spacing: u16,
}

pub(crate) fn modal_rect(area: Rect, size: ModalSize) -> Rect {
    let (percent_x, percent_y) = responsive_modal_percents(area, size);
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn responsive_modal_percents(area: Rect, size: ModalSize) -> (u16, u16) {
    let (mut percent_x, mut percent_y) = size.percents();
    if matches!(size, ModalSize::Compact) {
        if area.width <= 100 || area.height <= 34 {
            percent_x = percent_x.max(40);
            percent_y = percent_y.max(18);
        }
        if area.width <= 80 || area.height <= 26 {
            percent_x = percent_x.max(52);
            percent_y = percent_y.max(22);
        }
        return (percent_x, percent_y);
    }
    if area.width <= 100 || area.height <= 34 {
        percent_x = percent_x.max(92);
        percent_y = percent_y.max(90);
    }
    if area.width <= 80 || area.height <= 26 {
        percent_x = percent_x.max(96);
        percent_y = percent_y.max(94);
    }
    (percent_x, percent_y)
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use super::{ModalSize, modal_rect, responsive_modal_percents};

    #[test]
    fn compact_modal_stays_small_on_common_terminal_sizes() {
        let area = Rect::new(0, 0, 140, 40);
        let (width_pct, height_pct) = responsive_modal_percents(area, ModalSize::Compact);
        assert!(width_pct <= 48, "{width_pct}");
        assert!(height_pct <= 24, "{height_pct}");

        let rect = modal_rect(area, ModalSize::Compact);
        assert!(rect.width < 80, "rect: {rect:?}");
        assert!(rect.height < 12, "rect: {rect:?}");
    }
}

pub(crate) fn modal_size_for(kind: ModalLayoutKind) -> ModalSize {
    match kind {
        ModalLayoutKind::ToolDetail => ModalSize::Standard,
        ModalLayoutKind::Form => ModalSize::Wide,
        ModalLayoutKind::EditorSetup => ModalSize::Wide,
        ModalLayoutKind::Output => ModalSize::Wide,
        ModalLayoutKind::DebugLog => ModalSize::Wide,
        ModalLayoutKind::Help => ModalSize::Standard,
        ModalLayoutKind::CancelRunningConfirm => ModalSize::Compact,
        ModalLayoutKind::QuitConfirm => ModalSize::Compact,
    }
}

pub(crate) fn modal_content_min_height_for(area: Rect, kind: ModalLayoutKind) -> u16 {
    let base = match kind {
        ModalLayoutKind::ToolDetail => OUTPUT_CONTENT_MIN_HEIGHT,
        ModalLayoutKind::Form => FORM_CONTENT_MIN_HEIGHT,
        ModalLayoutKind::EditorSetup => EDITOR_SETUP_CONTENT_MIN_HEIGHT,
        ModalLayoutKind::Output => OUTPUT_CONTENT_MIN_HEIGHT,
        ModalLayoutKind::DebugLog => OUTPUT_CONTENT_MIN_HEIGHT,
        ModalLayoutKind::Help => OUTPUT_CONTENT_MIN_HEIGHT,
        ModalLayoutKind::CancelRunningConfirm => OUTPUT_CONTENT_MIN_HEIGHT,
        ModalLayoutKind::QuitConfirm => OUTPUT_CONTENT_MIN_HEIGHT,
    };
    if area.height <= 26 {
        base.saturating_sub(2).max(3)
    } else if area.height <= 34 {
        base.saturating_sub(1).max(3)
    } else {
        base
    }
}

pub(crate) fn modal_header_height_for(kind: ModalLayoutKind) -> u16 {
    match kind {
        ModalLayoutKind::Output => 0,
        ModalLayoutKind::DebugLog => 0,
        ModalLayoutKind::Form => 3,
        ModalLayoutKind::EditorSetup => 5,
        ModalLayoutKind::ToolDetail => 1,
        ModalLayoutKind::Help => 1,
        ModalLayoutKind::CancelRunningConfirm => 1,
        ModalLayoutKind::QuitConfirm => 1,
    }
}

pub(crate) fn page_shell_constraints(
    area: Rect,
    summary_height: u16,
    hint_height: u16,
) -> [Constraint; 7] {
    [
        Constraint::Length(TAB_BAR_HEIGHT),
        Constraint::Length(summary_height),
        Constraint::Length(page_summary_gap_for(area)),
        Constraint::Min(page_content_min_height_for(area)),
        Constraint::Length(STATUS_BAR_HEIGHT),
        Constraint::Length(hint_height),
        Constraint::Length(KEYS_BAR_HEIGHT),
    ]
}

pub(crate) fn page_summary_gap_for(area: Rect) -> u16 {
    if area.height <= 24 { 0 } else { 1 }
}

pub(crate) fn page_summary_max_height_for(area: Rect) -> u16 {
    if area.height <= 24 {
        2
    } else {
        PAGE_SUMMARY_MAX_HEIGHT
    }
}

pub(crate) fn page_content_min_height_for(area: Rect) -> u16 {
    if area.height <= 24 {
        5
    } else if area.height <= 32 {
        6
    } else {
        PAGE_CONTENT_MIN_HEIGHT
    }
}

pub(crate) fn hint_bar_max_height_for(area: Rect) -> u16 {
    if area.height <= 24 {
        2
    } else {
        HINT_BAR_MAX_HEIGHT
    }
}

pub(crate) fn modal_content_constraints(header_height: u16, content_min: u16) -> [Constraint; 5] {
    [
        Constraint::Length(header_height),
        Constraint::Min(content_min),
        Constraint::Length(MODAL_STATUS_HEIGHT),
        Constraint::Length(MODAL_HINT_HEIGHT),
        Constraint::Length(MODAL_KEYS_HEIGHT),
    ]
}

pub(crate) fn config_page_constraints(_area: Rect) -> [Constraint; 3] {
    [
        Constraint::Length(0),
        Constraint::Length(0),
        Constraint::Min(CONFIG_TARGETS_MIN_HEIGHT),
    ]
}

pub(crate) fn tools_table_layout_for(area: Rect) -> ToolsTableLayout {
    if area.width <= 96 {
        ToolsTableLayout {
            tool_width: 12,
            tag_width: 9,
            status_width: 10,
            version_width: 10,
            latest_width: 8,
            size_width: 9,
            column_spacing: 0,
        }
    } else if area.width <= 120 {
        ToolsTableLayout {
            tool_width: 14,
            tag_width: 9,
            status_width: 10,
            version_width: 12,
            latest_width: 10,
            size_width: 10,
            column_spacing: 1,
        }
    } else {
        ToolsTableLayout {
            tool_width: 16,
            tag_width: 10,
            status_width: 11,
            version_width: 14,
            latest_width: 12,
            size_width: 11,
            column_spacing: 1,
        }
    }
}

pub(crate) fn content_panel_padding_for(area: Rect) -> (u16, u16) {
    if area.width <= 90 || area.height <= 24 {
        (0, 0)
    } else {
        (CONTENT_PANEL_PADDING_X, CONTENT_PANEL_PADDING_Y)
    }
}

pub(crate) fn tools_table_constraints(layout: ToolsTableLayout) -> [Constraint; 9] {
    [
        Constraint::Length(4),
        Constraint::Length(layout.tool_width),
        Constraint::Length(layout.tag_width),
        Constraint::Length(layout.status_width),
        Constraint::Length(7),
        Constraint::Length(layout.version_width),
        Constraint::Length(layout.latest_width),
        Constraint::Length(layout.size_width),
        Constraint::Length(3),
    ]
}

pub(crate) fn modal_body_rect(area: Rect) -> Rect {
    let (pad_x, pad_y) = modal_body_padding_for(area);
    Rect {
        x: area.x.saturating_add(pad_x),
        y: area.y.saturating_add(pad_y),
        width: area.width.saturating_sub(pad_x.saturating_mul(2)),
        height: area.height.saturating_sub(pad_y.saturating_mul(2)),
    }
}

pub(crate) fn modal_body_rect_with_title_gap(area: Rect) -> Rect {
    let body = modal_body_rect(area);
    Rect {
        x: body.x,
        y: body.y.saturating_add(1),
        width: body.width,
        height: body.height.saturating_sub(1),
    }
}

pub(crate) fn modal_body_padding_for(area: Rect) -> (u16, u16) {
    if area.width <= 90 || area.height <= 24 {
        (0, 0)
    } else {
        (MODAL_BODY_PADDING_X, MODAL_BODY_PADDING_Y)
    }
}
