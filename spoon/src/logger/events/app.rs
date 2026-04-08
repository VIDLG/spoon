use std::path::Path;

pub fn session_start(path: &Path) {
    tracing::info!(path = %path.display(), "session.start");
}

pub fn app_start(command: impl AsRef<str>) {
    tracing::info!(command = command.as_ref(), "app.start");
}

pub fn tool_action_start(
    action: impl AsRef<str>,
    tools: impl IntoIterator<Item = impl AsRef<str>>,
) {
    let tools = tools
        .into_iter()
        .map(|tool| tool.as_ref().to_string())
        .collect::<Vec<_>>()
        .join(",");
    tracing::info!(action = action.as_ref(), tools = tools, "tool.action.start");
}
