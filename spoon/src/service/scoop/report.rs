use std::path::Path;

use crate::service::{CommandResult, CommandStatus, ConfigEntry, format_bytes};

use super::{
    ScoopPackageDetailsOutcome, command_result, installed_package_states, package_info,
    runtime_status, search_results,
};
use spoon_core::RuntimeLayout;

fn lines_or_default<T, F>(items: Vec<T>, empty: &str, map: F) -> Vec<String>
where
    F: FnMut(T) -> String,
{
    if items.is_empty() {
        vec![empty.to_string()]
    } else {
        items.into_iter().map(map).collect()
    }
}

fn section_lines<T, F>(title: &str, items: Vec<T>, empty: &str, mut map: F) -> Vec<String>
where
    F: FnMut(T) -> String,
{
    let mut lines = vec![format!("{title}:")];
    if items.is_empty() {
        lines.push(format!("  {empty}"));
    } else {
        lines.extend(items.into_iter().map(|item| format!("  {}", map(item))));
    }
    lines
}

pub async fn package_list_report(tool_root: &Path) -> CommandResult {
    let packages = installed_package_states(tool_root)
        .await
        .into_iter()
        .map(
            |state| spoon_scoop::InstalledPackageSummary {
                name: state.identity.package,
                version: state.identity.version.trim().to_string(),
            },
        )
        .collect::<Vec<_>>();
    let output = lines_or_default(
        packages,
        "No Scoop packages are currently installed.",
        |package| format!("{} | {}", package.name, package.version),
    );
    command_result("list Scoop packages", CommandStatus::Success, output, false)
}

pub async fn package_prefix_report(tool_root: &Path, package_name: &str) -> CommandResult {
    let layout = RuntimeLayout::from_root(tool_root);
    let prefix = layout.scoop.apps_root.join(package_name).join("current");
    let status_data = runtime_status(tool_root).await;
    let installed_version = status_data
        .installed_packages
        .iter()
        .find(|p| p.name == package_name)
        .map(|p| p.version.trim().to_string());
    let installed = installed_version.is_some() && prefix.exists();
    let mut output = Vec::new();
    if installed {
        output.push(prefix.display().to_string());
    } else {
        output.push(format!("Scoop package '{package_name}' is not installed."));
    }
    let status = if installed {
        CommandStatus::Success
    } else {
        CommandStatus::Failed
    };
    command_result(
        format!("prefix Scoop package {package_name}"),
        status,
        output,
        false,
    )
}

pub async fn runtime_status_report(tool_root: &Path) -> CommandResult {
    let data = runtime_status(tool_root).await;
    let mut output = vec![
        "Scoop runtime:".to_string(),
        format!("  root: {}", data.runtime.root),
        format!("  shims: {}", data.runtime.shims),
        format!("  buckets: {}", data.buckets.len()),
        format!("  installed packages: {}", data.installed_packages.len()),
    ];
    output.extend(section_lines("Buckets", data.buckets, "none", |bucket| {
        format!("{} | {} | {}", bucket.name, bucket.branch, bucket.source)
    }));
    output.extend(section_lines(
        "Installed packages",
        data.installed_packages,
        "none",
        |package| format!("{} | {}", package.name, package.version),
    ));
    output.push("Paths:".to_string());
    output.push(format!("  apps: {}", data.paths.apps));
    output.push(format!("  cache: {}", data.paths.cache));
    output.push(format!("  persist: {}", data.paths.persist));
    output.push(format!("  state: {}", data.paths.state));
    command_result(
        "status Scoop runtime",
        CommandStatus::Success,
        output,
        false,
    )
}

pub async fn search_report(tool_root: &Path, query: Option<&str>) -> CommandResult {
    let data = search_results(tool_root, query).await;
    let output = lines_or_default(data.matches, "No matching Scoop packages found.", |item| {
        format!(
            "{} | {} | {} | {}",
            item.package_name,
            item.version.unwrap_or_else(|| "-".to_string()),
            item.bucket,
            item.description.unwrap_or_default()
        )
    });
    let title = match data.query {
        Some(query) => format!("search Scoop packages for {query}"),
        None => "search Scoop packages".to_string(),
    };
    command_result(title, CommandStatus::Success, output, false)
}

pub async fn package_info_report(tool_root: &Path, package_name: &str) -> CommandResult {
    match package_info(tool_root, package_name).await {
        ScoopPackageDetailsOutcome::Details(details) => {
            let mut output = format_package_section(details.package);
            output.push(String::new());
            output.extend(format_install_section(details.install));
            let integration_lines = format_integration_section(details.integration);
            if !integration_lines.is_empty() {
                output.push(String::new());
                output.push("Integration:".to_string());
                output.extend(integration_lines);
            }

            command_result(
                format!("info Scoop package {package_name}"),
                CommandStatus::Success,
                output,
                false,
            )
        }
        ScoopPackageDetailsOutcome::Error(error) => command_result(
            format!("info Scoop package {}", error.package),
            CommandStatus::Failed,
            vec![error.error.message],
            false,
        ),
    }
}

fn format_package_section(package: spoon_scoop::ScoopPackageMetadata) -> Vec<String> {
    let mut output = vec![
        "Package:".to_string(),
        format!("  name: {}", package.name),
        format!("  bucket: {}", package.bucket),
        format!(
            "  latest version: {}",
            package.latest_version.as_deref().unwrap_or("-")
        ),
        format!(
            "  description: {}",
            package.description.as_deref().unwrap_or("-")
        ),
        format!("  homepage: {}", package.homepage.as_deref().unwrap_or("-")),
        format!("  manifest: {}", package.manifest),
    ];
    if let Some(license) = package.license {
        output.push(format!("  license: {license}"));
    }
    for (label, value) in [
        ("depends", package.depends),
        ("suggest", package.suggest),
        ("extract dir", package.extract_dir),
        ("extract to", package.extract_to),
    ] {
        if let Some(value) = value {
            output.push(format!("  {label}: {value}"));
        }
    }
    for note in package.notes {
        if note.is_empty() {
            output.push(String::new());
        } else {
            output.push(format!("  {note}"));
        }
    }
    for url in package.download_urls {
        output.push(format!("  download url: {url}"));
    }
    output
}

fn format_install_section(install: spoon_scoop::ScoopPackageInstall) -> Vec<String> {
    let mut output = vec![
        "Install:".to_string(),
        format!(
            "  installed: {}",
            if install.installed { "yes" } else { "no" }
        ),
        format!(
            "  installed version: {}",
            install.installed_version.as_deref().unwrap_or("-")
        ),
        format!("  current: {}", install.current),
    ];
    if let Some(bytes) = install.installed_size_bytes {
        output.push(format!("  installed size: {}", format_bytes(bytes)));
    }
    if let Some(bytes) = install.cache_size_bytes {
        output.push(format!("  cache size: {}", format_bytes(bytes)));
    }
    for bin in install.bins {
        output.push(format!("  bin: {bin}"));
    }
    if let Some(state) = install.state {
        output.push(format!("  state: {state}"));
    }
    if let Some(persist_root) = install.persist_root {
        output.push(format!("  persist root: {persist_root}"));
    }
    output
}

fn format_integration_section(
    integration: spoon_scoop::ScoopPackageIntegration<ConfigEntry>,
) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(shims) = integration.commands.shims
        && !shims.is_empty()
    {
        lines.push("  Commands:".to_string());
        lines.push(format!("    shims: {}", shims.join(", ")));
    }
    if !integration.environment.add_path.is_empty()
        || !integration.environment.set.is_empty()
        || integration.environment.persist.is_some()
    {
        lines.push("  Environment:".to_string());
        for value in integration.environment.add_path {
            lines.push(format!("    add_path: {value}"));
        }
        for value in integration.environment.set {
            lines.push(format!("    set: {value}"));
        }
        if let Some(value) = integration.environment.persist {
            lines.push(format!("    persist: {value}"));
        }
    }
    if !integration.system.shortcuts.is_empty() {
        lines.push("  System:".to_string());
        for shortcut in integration.system.shortcuts {
            lines.push(format!("    {shortcut}"));
        }
    }
    if !integration.policy.desired.is_empty()
        || !integration.policy.applied_values.is_empty()
        || !integration.policy.config_files.is_empty()
        || !integration.policy.config_directories.is_empty()
    {
        lines.push("  Policy:".to_string());
        for entry in integration.policy.desired {
            lines.push(format!(
                "    desired: {}: {}",
                entry.key,
                entry.value.display_value()
            ));
        }
        for value in integration.policy.applied_values {
            lines.push(format!("    applied value: {}: {}", value.key, value.value));
        }
        for value in integration.policy.config_files {
            lines.push(format!("    config file: {value}"));
        }
        for value in integration.policy.config_directories {
            lines.push(format!("    config directory: {value}"));
        }
    }
    lines
}

pub async fn package_manifest(tool_root: &Path, package_name: &str) -> CommandResult {
    let outcome = spoon_scoop::package_manifest(tool_root, package_name).await;
    let output = match (outcome.content, outcome.error) {
        (Some(content), _) => content.lines().map(str::to_string).collect(),
        (None, Some(error)) => vec![error.message],
        (None, None) => Vec::new(),
    };
    command_result(outcome.title, outcome.status, output, outcome.streamed)
}
