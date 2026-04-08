pub fn tui_background_status_check_start() {
    tracing::info!("tui.background.status_check.start");
}

pub fn tui_background_status_check_update(statuses: usize, has_update_info: bool) {
    tracing::info!(
        statuses = statuses,
        update_info = has_update_info,
        "tui.background.status_check.update"
    );
}

pub fn tui_background_status_check_complete() {
    tracing::info!("tui.background.status_check.complete");
}

pub fn tui_background_status_check_disconnected() {
    tracing::warn!("tui.background.status_check.disconnected");
}

pub fn tui_background_action_complete(title: &str, status: &str) {
    let lower = status.to_ascii_lowercase();
    if lower.contains("failed") || lower.contains("error") {
        tracing::error!(
            title = title,
            status = status,
            "tui.background.action.complete"
        );
    } else if lower.contains("blocked") || lower.contains("missing") || lower.contains("warning") {
        tracing::warn!(
            title = title,
            status = status,
            "tui.background.action.complete"
        );
    } else {
        tracing::info!(
            title = title,
            status = status,
            "tui.background.action.complete"
        );
    }
}

pub fn tui_background_action_disconnected() {
    tracing::warn!("tui.background.action.disconnected");
}

pub fn tui_tools_action_start(action: &str, tools: impl IntoIterator<Item = impl AsRef<str>>) {
    let tools = tools
        .into_iter()
        .map(|tool| tool.as_ref().to_string())
        .collect::<Vec<_>>()
        .join(",");
    tracing::info!(action = action, tools = tools, "tui.tools.action.start");
}
