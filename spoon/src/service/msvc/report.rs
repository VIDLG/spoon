pub(super) fn status_report_lines(data: spoon_msvc::status::MsvcStatus) -> Vec<String> {
    let mut output = vec!["MSVC runtimes:".to_string(), "Managed:".to_string()];
    output.push(format!("  status: {}", data.managed.status));
    output.push(format!("  root: {}", data.managed.root));
    output.push(format!("  toolchain: {}", data.managed.toolchain));
    output.push(format!("  state: {}", data.managed.state));
    output.push(format!("  cache: {}", data.managed.cache));
    output.push(format!(
        "  runtime.json: {}",
        runtime_state_label(data.managed.runtime_state_present)
    ));
    output.push(format!("  archives: {}", data.managed.archives));
    output.push(format!(
        "  staged MSI payloads: {}",
        data.managed.staged_msi_payloads
    ));
    output.push(format!(
        "  extracted MSI payloads: {}",
        data.managed.extracted_msi_payloads
    ));
    output.push(format!(
        "  install image files: {}",
        data.managed.install_image_files
    ));
    output.push("  Integration:".to_string());
    output.extend(managed_integration_lines(data.managed.integration));

    output.push("Official:".to_string());
    output.push(format!("  status: {}", data.official.status));
    output.push(format!("  root: {}", data.official.root));
    output.push(format!("  state: {}", data.official.state));
    output.push(format!("  cache: {}", data.official.cache));
    output.push(format!(
        "  runtime.json: {}",
        runtime_state_label(data.official.runtime_state_present)
    ));
    output.push("  Integration:".to_string());
    output.extend(official_integration_lines(data.official.integration));
    output
}

fn runtime_state_label(present: bool) -> &'static str {
    if present { "present" } else { "missing" }
}

fn managed_integration_lines(integration: spoon_msvc::status::MsvcIntegration) -> Vec<String> {
    match integration {
        spoon_msvc::status::MsvcIntegration::ActiveManaged(integration) => vec![
            "    Commands:".to_string(),
            format!(
                "      wrappers: {}",
                if integration.commands.wrappers.is_empty() {
                    "none materialized".to_string()
                } else {
                    integration.commands.wrappers.join(", ")
                }
            ),
            "    Environment:".to_string(),
            format!("      shims root: {}", integration.environment.shims_root),
            format!(
                "      user PATH entry: {}",
                integration.environment.user_path_entry
            ),
        ],
        _ => vec!["    none yet".to_string()],
    }
}

fn official_integration_lines(integration: spoon_msvc::status::MsvcIntegration) -> Vec<String> {
    match integration {
        spoon_msvc::status::MsvcIntegration::ActiveOfficial(integration) => vec![
            "    System:".to_string(),
            format!(
                "      vswhere discovery: {}",
                integration.system.vswhere_discovery
            ),
            format!(
                "      shared Windows SDK root: {}",
                integration.system.shared_windows_sdk_root
            ),
            format!("      registration: {}", integration.system.registration),
        ],
        _ => vec!["    none yet".to_string()],
    }
}
