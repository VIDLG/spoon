use crate::service::CommandResult;

#[derive(Debug, Clone, Copy)]
pub struct EventScope {
    pub layer: &'static str,
    pub domain: &'static str,
    pub action: &'static str,
}

impl EventScope {
    pub const fn new(layer: &'static str, domain: &'static str, action: &'static str) -> Self {
        Self {
            layer,
            domain,
            action,
        }
    }
}

pub const CLI_MSVC_VALIDATE: EventScope = EventScope::new("cli", "msvc", "validate");
pub const CLI_MSVC_ACTION: EventScope = EventScope::new("cli", "msvc", "action");
pub const CLI_MSVC_STATUS: EventScope = EventScope::new("cli", "msvc", "status");
pub const CLI_SCOOP_PACKAGE_ACTION: EventScope = EventScope::new("cli", "scoop", "package_action");
pub const CLI_SCOOP_DOCTOR: EventScope = EventScope::new("cli", "scoop", "doctor");
pub const CLI_SCOOP_SEARCH: EventScope = EventScope::new("cli", "scoop", "search");
pub const CLI_SCOOP_PACKAGE_QUERY: EventScope = EventScope::new("cli", "scoop", "package_query");
pub const CLI_SCOOP_BUCKET_ACTION: EventScope = EventScope::new("cli", "scoop", "bucket_action");
pub const CLI_SCOOP_STATUS: EventScope = EventScope::new("cli", "scoop", "status");
pub const TOOL_ACTION_RESULT: EventScope = EventScope::new("service", "tools", "result");
pub const EDITOR_ACTION_RESULT: EventScope = EventScope::new("service", "editor", "result");

fn command_result(scope: EventScope, title: &str, result: &CommandResult) {
    let success = result.is_success();
    if success {
        tracing::info!(
            layer = scope.layer,
            domain = scope.domain,
            action = scope.action,
            title = title,
            status = result.status.as_str(),
            success = true,
            "command.result"
        );
    } else {
        tracing::error!(
            layer = scope.layer,
            domain = scope.domain,
            action = scope.action,
            title = title,
            status = result.status.as_str(),
            success = false,
            "command.result"
        );
    }
    for line in &result.output {
        if success {
            tracing::info!(
                layer = scope.layer,
                domain = scope.domain,
                action = scope.action,
                title = title,
                line = line,
                "command.result.line"
            );
        } else {
            tracing::error!(
                layer = scope.layer,
                domain = scope.domain,
                action = scope.action,
                title = title,
                line = line,
                "command.result.line"
            );
        }
    }
}

pub fn command_results(scope: EventScope, results: &[CommandResult]) {
    tracing::info!(
        layer = scope.layer,
        domain = scope.domain,
        action = scope.action,
        results = results.len(),
        "command.results"
    );
    for result in results {
        command_result(scope, &result.title, result);
    }
}
