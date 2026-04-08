use crate::service::CommandResult;

pub fn flatten_command_results(results: Vec<CommandResult>) -> Vec<String> {
    flatten_filtered_command_results(results, false)
}

pub fn flatten_unstreamed_command_results(results: Vec<CommandResult>) -> Vec<String> {
    flatten_filtered_command_results(results, true)
}

fn flatten_filtered_command_results(
    results: Vec<CommandResult>,
    only_unstreamed: bool,
) -> Vec<String> {
    let mut lines = Vec::new();
    for result in results {
        if only_unstreamed && result.streamed {
            continue;
        }
        lines.push(format!("== {} ==", result.title));
        if result.output.is_empty() {
            lines.push("(no output)".to_string());
        } else {
            lines.extend(result.output);
        }
        lines.push(String::new());
    }
    lines
}

pub fn summarize_command_results(results: Vec<CommandResult>) -> (Vec<String>, String) {
    let status = summarize_command_status(&results);
    (flatten_command_results(results), status)
}

pub fn summarize_streamed_command_results(results: Vec<CommandResult>) -> (Vec<String>, String) {
    let status = summarize_command_status(&results);
    (flatten_unstreamed_command_results(results), status)
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
