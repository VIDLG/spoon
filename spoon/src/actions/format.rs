use crate::service::CommandResult;

pub fn flatten_command_results(results: Vec<CommandResult>) -> Vec<String> {
    let mut lines = Vec::new();
    for result in results {
        lines.push(format!("== {} ==", result.title));
        lines.push(String::new());
    }
    lines
}

/// Kept for backward compat with callers that previously distinguished streamed/unstreamed.
/// Now equivalent to flatten_command_results since streamed distinction is removed.
pub fn flatten_unstreamed_command_results(results: Vec<CommandResult>) -> Vec<String> {
    flatten_command_results(results)
}

pub fn summarize_command_results(results: Vec<CommandResult>) -> (Vec<String>, String) {
    let status = summarize_command_status(&results);
    (flatten_command_results(results), status)
}

/// Kept for backward compat with callers that previously distinguished streamed/unstreamed.
/// Now equivalent to summarize_command_results since streamed distinction is removed.
pub fn summarize_streamed_command_results(results: Vec<CommandResult>) -> (Vec<String>, String) {
    summarize_command_results(results)
}

pub fn summarize_command_status(results: &[CommandResult]) -> String {
    if results
        .iter()
        .any(|r| matches!(r.status, crate::service::CommandStatus::Cancelled))
    {
        return "action cancelled".to_string();
    }
    if results
        .iter()
        .any(|r| matches!(r.status, crate::service::CommandStatus::Blocked))
    {
        return "blocked by prerequisites".to_string();
    }
    if results.iter().any(|r| !r.is_success()) {
        return "action failed".to_string();
    }
    "action completed".to_string()
}
