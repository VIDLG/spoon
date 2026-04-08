use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::{Value, json};

use crate::actions::ToolAction;
use crate::config;
use crate::logger;
use crate::packages::{
    self, PackageConfigImportResult, PackageConfigReapply, PackageConfigSetResult,
};
use crate::bridge::StreamChunk;
use crate::bridge::{CacheScope, cache_action_result, clear_cache, msvc, prune_cache, scoop};
use crate::status;
use crate::view;

use super::{
    Commands, ConfigRootCommand, ConfigScopeCommand, ConfigSubcommand, DomainCacheSubcommand,
    MsvcInstallerModeArg, MsvcRuntimeArg, MsvcRuntimeCommand, MsvcSubcommand, MsvcValidateCommand,
    ScoopBucketSubcommand, ScoopPackageCommand, ScoopSearchCommand, ScoopSinglePackageCommand,
    ScoopSubcommand, StatusCommand, json as cli_json, messages, output,
};

fn effective_root(install_root: Option<&Path>) -> Option<PathBuf> {
    install_root
        .map(Path::to_path_buf)
        .or_else(config::configured_tool_root)
}

fn print_command_result(result: &crate::bridge::CommandResult, json_mode: bool) {
    if json_mode {
        output::print_json_value(&cli_json::command_result(result));
    }
    // Non-streamed output lines are no longer stored in CommandResult.
    // Streaming output is forwarded in real-time via the FnMut(StreamChunk) callback.
}

fn print_cli_response(response: &crate::cli::response::CliResponse, json_mode: bool) {
    if json_mode {
        output::print_json_value(&cli_json::cli_response(response));
    } else {
        output::print_response(response);
    }
}

fn print_config_view(json_mode: bool) {
    if json_mode {
        output::print_json_value(&cli_json::config_view());
    } else {
        let model = view::build_config_model();
        output::print_response(&messages::config_view(&model));
    }
}

fn print_config_path(json_mode: bool) {
    if json_mode {
        output::print_json_value(&cli_json::config_path());
    } else {
        output::print_lines(&[config::global_config_path().display().to_string()]);
    }
}

fn print_config_cat(json_mode: bool) -> Result<()> {
    let path = config::global_config_path();
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    if json_mode {
        output::print_json_value(&cli_json::config_cat(&path, &content));
    } else {
        let lines = content.lines().map(str::to_string).collect::<Vec<_>>();
        output::print_toml_lines(&lines);
    }
    Ok(())
}

fn print_package_config_view(package_key: &str, json_mode: bool) {
    if json_mode {
        if let Some(view) = cli_json::config_scope_view(package_key) {
            output::print_json_value(&view);
        }
    } else if let Some(model) = view::build_package_config_scope_model(package_key) {
        output::print_response(&messages::config_scope_view(&model));
    }
}

fn parse_config_assignment(
    key: Option<String>,
    value: Option<String>,
) -> (Option<String>, Option<String>) {
    match (key, value) {
        (Some(key), None) => {
            if let Some((parsed_key, parsed_value)) = key.split_once('=') {
                let parsed_key = parsed_key.trim();
                let parsed_value = parsed_value.trim();
                if !parsed_key.is_empty() {
                    return (Some(parsed_key.to_string()), Some(parsed_value.to_string()));
                }
            }
            (Some(key), None)
        }
        other => other,
    }
}

fn print_config_scope_result(
    scope: &'static str,
    action: &'static str,
    changed_key: Option<&str>,
    changed_value: Option<&str>,
    reapply_output: &[String],
    json_mode: bool,
) {
    if json_mode {
        let view = cli_json::config_scope_view(scope)
            .map(|view| serde_json::to_value(&view).expect("config scope json"))
            .unwrap_or_else(|| json!(null));
        output::print_json_value(&json!({
            "kind": "config_scope_result",
            "scope": scope,
            "action": action,
            "changed_key": changed_key,
            "changed_value": changed_value,
            "reapply_output": reapply_output,
            "view": view.get("data").cloned().unwrap_or(Value::Null),
        }));
    } else {
        print_package_config_view(scope, false);
    }
}

async fn run_package_config_command(
    ConfigScopeCommand { key, value }: ConfigScopeCommand,
    package_key: &'static str,
    json_mode: bool,
) -> Result<()> {
    let (key, value) = parse_config_assignment(key, value);
    let Some(key) = key else {
        print_package_config_view(package_key, json_mode);
        return Ok(());
    };

    if key == "import" {
        match packages::import_config(package_key)? {
            Some(PackageConfigImportResult::Changed(mutation)) => {
                if !json_mode {
                    print_cli_response(
                        &messages::config_imported(&mutation.changed_key, &mutation.changed_value),
                        false,
                    );
                }
                let reapply_output =
                    reapply_package_config_change(package_key, mutation.reapply, json_mode).await?;
                print_config_scope_result(
                    package_key,
                    "import",
                    Some(&mutation.changed_key),
                    Some(&mutation.changed_value),
                    &reapply_output,
                    json_mode,
                );
            }
            Some(PackageConfigImportResult::Skipped { reason }) => {
                print_cli_response(
                    &messages::config_import_skipped(package_key, &reason),
                    json_mode,
                );
                print_package_config_view(package_key, json_mode);
            }
            None => {
                print_cli_response(
                    &messages::unknown_config_key(package_key, "import"),
                    json_mode,
                );
            }
        }
        return Ok(());
    }

    let Some(value) = value else {
        print_cli_response(
            &messages::missing_config_value(package_key, Some(&key)),
            json_mode,
        );
        return Ok(());
    };

    match packages::set_config_value(package_key, &key, &value)? {
        PackageConfigSetResult::Changed(mutation) => {
            if !json_mode {
                print_cli_response(
                    &messages::config_updated(&mutation.changed_key, &mutation.changed_value),
                    false,
                );
            }
            let reapply_output =
                reapply_package_config_change(package_key, mutation.reapply, json_mode).await?;
            print_config_scope_result(
                package_key,
                "set",
                Some(&mutation.changed_key),
                Some(&mutation.changed_value),
                &reapply_output,
                json_mode,
            );
        }
        PackageConfigSetResult::UnknownKey => {
            print_cli_response(&messages::unknown_config_key(package_key, &key), json_mode);
        }
        PackageConfigSetResult::InvalidValue { expected } => {
            print_cli_response(
                &messages::invalid_config_value(&format!("{package_key}.{key}"), &value, expected),
                json_mode,
            );
        }
    }
    Ok(())
}

async fn reapply_package_config_change(
    package_key: &str,
    reapply: PackageConfigReapply,
    json_mode: bool,
) -> Result<Vec<String>> {
    let Some(root) = config::configured_tool_root() else {
        return Ok(Vec::new());
    };
    match reapply {
        PackageConfigReapply::None => Ok(Vec::new()),
        PackageConfigReapply::ScoopIntegrations => {
            if json_mode {
                scoop::reapply_package_integrations(&root, package_key).await
            } else {
                scoop::reapply_package_integrations_with_emit(&root, package_key, output::print_stream_chunk).await
            }
        }
        PackageConfigReapply::ScoopCommandSurface => {
            if json_mode {
                scoop::reapply_package_command_surface(&root, package_key).await
            } else {
                scoop::reapply_package_command_surface_with_emit(&root, package_key, output::print_stream_chunk).await
            }
        }
        PackageConfigReapply::ManagedMsvcCommandSurface => {
            let command_profile = crate::config::load_policy_config().msvc.command_profile;
            if json_mode {
                msvc::reapply_managed_command_surface(&root, &command_profile).await
            } else {
                msvc::reapply_managed_command_surface_with_emit(&root, &command_profile, output::print_stream_chunk).await
            }
        }
    }
}

fn run_root_config_command(
    ConfigRootCommand { value }: ConfigRootCommand,
    cli_root: Option<PathBuf>,
    json_mode: bool,
) -> Result<()> {
    let mut global = config::load_global_config();
    if let Some(root) = value.or(cli_root) {
        global.root = root.display().to_string();
        config::save_global_config(&global)?;
        logger::config_root_set(&global.root);
        if json_mode {
            output::print_json_value(&json!({
                "kind": "config_root_result",
                "action": "set",
                "root": global.root,
                "view": cli_json::config_view().data,
            }));
        } else {
            output::print_response(&messages::config_root_updated(&global.root));
        }
    } else if global.root.trim().is_empty() {
        logger::config_root_unset();
        print_cli_response(&messages::config_root_unset(), json_mode);
    } else {
        logger::config_root_set(&global.root);
        if json_mode {
            output::print_json_value(&json!({
                "kind": "config_root_result",
                "action": "show",
                "root": global.root,
                "view": cli_json::config_view().data,
            }));
        } else {
            output::print_response(&messages::config_root_updated(&global.root));
        }
    }
    Ok(())
}

fn selected_installer_mode(
    mode: MsvcInstallerModeArg,
    passive: bool,
    quiet: bool,
) -> MsvcInstallerModeArg {
    if quiet {
        MsvcInstallerModeArg::Quiet
    } else if passive {
        MsvcInstallerModeArg::Passive
    } else {
        mode
    }
}

fn installed_msvc_runtimes(root: &Path) -> Vec<MsvcRuntimeArg> {
    let mut runtimes = Vec::new();
    if msvc::runtime_state_path(root).exists() {
        runtimes.push(MsvcRuntimeArg::Managed);
    }
    if msvc::official::runtime_state_path(root).exists() {
        runtimes.push(MsvcRuntimeArg::Official);
    }
    runtimes
}

async fn run_msvc_validation(
    runtime: Option<MsvcRuntimeArg>,
    install_root: Option<&Path>,
    json_mode: bool,
) -> Result<()> {
    let Some(root) = install_root else {
        print_cli_response(&messages::missing_msvc_root(), json_mode);
        return Ok(());
    };
    let runtimes = if let Some(runtime) = runtime {
        vec![runtime]
    } else {
        installed_msvc_runtimes(root)
    };
    if runtimes.is_empty() {
        print_cli_response(&messages::no_installed_msvc_runtimes(), json_mode);
        return Ok(());
    }
    for runtime in runtimes {
        match runtime {
            MsvcRuntimeArg::Managed => {
                let result = msvc::validate_toolchain(root).await?;
                logger::command_results(logger::CLI_MSVC_VALIDATE, std::slice::from_ref(&result));
                print_command_result(&result, json_mode);
            }
            MsvcRuntimeArg::Official => {
                let result = msvc::official::validate_toolchain(root).await?;
                logger::command_results(logger::CLI_MSVC_VALIDATE, std::slice::from_ref(&result));
                print_command_result(&result, json_mode);
            }
        }
    }
    Ok(())
}

async fn run_msvc_action(
    action: ToolAction,
    runtime: MsvcRuntimeArg,
    mode: MsvcInstallerModeArg,
    install_root: Option<&Path>,
    json_mode: bool,
) -> Result<()> {
    if matches!(runtime, MsvcRuntimeArg::Official) {
        let installer_mode = match mode {
            MsvcInstallerModeArg::Quiet => msvc::official::OfficialInstallerMode::Quiet,
            MsvcInstallerModeArg::Passive => msvc::official::OfficialInstallerMode::Passive,
        };
        let result = match action {
            ToolAction::Install => match install_root {
                Some(root) => {
                    if json_mode {
                        msvc::official::install_toolchain(root, installer_mode, None, None).await?
                    } else {
                        msvc::official::install_toolchain_with_emit(root, installer_mode, None, output::print_stream_chunk).await?
                    }
                }
                None => {
                    print_cli_response(&messages::missing_msvc_root(), json_mode);
                    return Ok(());
                }
            },
            ToolAction::Update => match install_root {
                Some(root) => {
                    if json_mode {
                        msvc::official::update_toolchain(root, installer_mode, None, None).await?
                    } else {
                        msvc::official::update_toolchain_with_emit(root, installer_mode, None, output::print_stream_chunk).await?
                    }
                }
                None => {
                    print_cli_response(&messages::missing_msvc_root(), json_mode);
                    return Ok(());
                }
            },
            ToolAction::Uninstall => match install_root {
                Some(root) => {
                    if json_mode {
                        msvc::official::uninstall_toolchain(root, installer_mode, None, None).await?
                    } else {
                        msvc::official::uninstall_toolchain_with_emit(root, installer_mode, None, output::print_stream_chunk).await?
                    }
                }
                None => {
                    print_cli_response(&messages::missing_msvc_root(), json_mode);
                    return Ok(());
                }
            },
        };
        logger::command_results(logger::CLI_MSVC_ACTION, std::slice::from_ref(&result));
        print_command_result(&result, json_mode);
        return Ok(());
    }
    let result = match action {
        ToolAction::Install => match install_root {
            Some(root) => {
                if json_mode {
                    msvc::install_toolchain(root, None, None).await?
                } else {
                    msvc::install_toolchain_with_emit(root, None, output::print_stream_chunk).await?
                }
            }
            None => {
                print_cli_response(&messages::missing_msvc_root(), json_mode);
                return Ok(());
            }
        },
        ToolAction::Update => match install_root {
            Some(root) => {
                if json_mode {
                    msvc::update_toolchain(root, None, None).await?
                } else {
                    msvc::update_toolchain_with_emit(root, None, output::print_stream_chunk).await?
                }
            }
            None => {
                print_cli_response(&messages::missing_msvc_root(), json_mode);
                return Ok(());
            }
        },
        ToolAction::Uninstall => match install_root {
            Some(root) => msvc::uninstall_toolchain(root, None, None).await?,
            None => {
                print_cli_response(&messages::missing_msvc_root(), json_mode);
                return Ok(());
            }
        },
    };
    logger::command_results(logger::CLI_MSVC_ACTION, std::slice::from_ref(&result));
    print_command_result(&result, json_mode);
    Ok(())
}

async fn run_msvc_command(
    command: MsvcSubcommand,
    install_root: Option<&Path>,
    json_mode: bool,
) -> Result<()> {
    match command {
        MsvcSubcommand::Status => {
            let Some(root) = install_root else {
                print_cli_response(&messages::missing_msvc_root(), json_mode);
                return Ok(());
            };
            if json_mode {
                output::print_json_value(&msvc::status(root).await);
                return Ok(());
            }
            let result = msvc::status_report(root).await;
            let lines = msvc::status_report_lines(root).await;
            logger::command_results(logger::CLI_MSVC_STATUS, std::slice::from_ref(&result));
            output::print_lines(&lines);
            Ok(())
        }
        MsvcSubcommand::Install(MsvcRuntimeCommand {
            runtime,
            mode,
            passive,
            quiet,
        }) => {
            run_msvc_action(
                ToolAction::Install,
                runtime,
                selected_installer_mode(mode, passive, quiet),
                install_root,
                json_mode,
            )
            .await
        }
        MsvcSubcommand::Update(MsvcRuntimeCommand {
            runtime,
            mode,
            passive,
            quiet,
        }) => {
            run_msvc_action(
                ToolAction::Update,
                runtime,
                selected_installer_mode(mode, passive, quiet),
                install_root,
                json_mode,
            )
            .await
        }
        MsvcSubcommand::Uninstall(MsvcRuntimeCommand {
            runtime,
            mode,
            passive,
            quiet,
        }) => {
            run_msvc_action(
                ToolAction::Uninstall,
                runtime,
                selected_installer_mode(mode, passive, quiet),
                install_root,
                json_mode,
            )
            .await
        }
        MsvcSubcommand::Validate(MsvcValidateCommand { runtime }) => {
            run_msvc_validation(runtime, install_root, json_mode).await
        }
        MsvcSubcommand::Cache { command } => {
            let Some(root) = install_root else {
                print_cli_response(&messages::missing_msvc_root(), json_mode);
                return Ok(());
            };
            run_domain_cache_command(command, CacheScope::Msvc, root, json_mode);
            Ok(())
        }
    }
}

fn run_domain_cache_command(
    command: DomainCacheSubcommand,
    scope: CacheScope,
    root: &Path,
    json_mode: bool,
) {
    let roots = crate::bridge::cache_roots_for_tool_root(root);
    let (action, result, lines) = match command {
        DomainCacheSubcommand::Prune => {
            let lines = crate::bridge::cache_prune_lines(&roots, scope)
                .expect("cache prune should succeed once root is configured");
            ("prune", prune_cache(root, scope), lines)
        }
        DomainCacheSubcommand::Clear => {
            let lines = crate::bridge::cache_clear_lines(&roots, scope)
                .expect("cache clear should succeed once root is configured");
            ("clear", clear_cache(root, scope), lines)
        }
    };
    let result = result.expect("cache command should succeed once root is configured");
    if json_mode {
        output::print_json_value(&cache_action_result(root, scope, action, &result));
    } else {
        output::print_lines(&lines);
    }
}

fn run_scoop_package_command(
    action: &str,
    command: &ScoopPackageCommand,
    install_root: Option<&Path>,
    json_mode: bool,
) -> Result<()> {
    let effective_root = effective_root(install_root);
    let Some(root) = effective_root.as_deref() else {
        print_cli_response(&messages::missing_scoop_root(), json_mode);
        return Ok(());
    };
    if command.packages.is_empty() {
        print_cli_response(&messages::no_scoop_packages_selected(), json_mode);
        return Ok(());
    }
    let mut json_results = Vec::new();
    for package in &command.packages {
        let display_name = package.replace('-', " ");
        let result = scoop::run_package_action_streaming(
            action,
            &display_name,
            package,
            Some(root),
            None,
            if json_mode {
                Option::<fn(StreamChunk)>::None
            } else {
                Some(output::print_stream_chunk)
            },
        )?;
        logger::command_results(
            logger::CLI_SCOOP_PACKAGE_ACTION,
            std::slice::from_ref(&result),
        );
        if json_mode {
            json_results.push(scoop::package_action_result(
                root,
                action,
                package,
                &display_name,
                &result,
            )?);
        } else {
            print_command_result(&result, false);
        }
    }
    if json_mode {
        output::print_json_value(&json!({
            "kind": "scoop_package_actions",
            "action": action,
            "results": json_results,
        }));
    }
    Ok(())
}

async fn run_scoop_command(
    command: ScoopSubcommand,
    install_root: Option<&Path>,
    json_mode: bool,
) -> Result<()> {
    match command {
        ScoopSubcommand::Status => {
            let effective_root = effective_root(install_root);
            let Some(root) = effective_root.as_deref() else {
                print_cli_response(&messages::missing_scoop_root(), json_mode);
                return Ok(());
            };
            if json_mode {
                output::print_json_value(&scoop::runtime_status(root).await);
                return Ok(());
            }
            let result = scoop::runtime_status_report(root).await;
            let lines = scoop::runtime_status_report_lines(root).await;
            logger::command_results(logger::CLI_SCOOP_STATUS, std::slice::from_ref(&result));
            output::print_lines(&lines);
            Ok(())
        }
        ScoopSubcommand::List => {
            let effective_root = effective_root(install_root);
            let Some(root) = effective_root.as_deref() else {
                print_cli_response(&messages::missing_scoop_root(), json_mode);
                return Ok(());
            };
            if json_mode {
                let packages = scoop::installed_package_states(root)
                    .await
                    .into_iter()
                    .map(
                        |state| spoon_scoop::InstalledPackageSummary {
                            name: state.identity.package,
                            version: state.identity.version.trim().to_string(),
                        },
                    )
                    .collect::<Vec<_>>();
                output::print_json_value(&json!({
                    "kind": "scoop_package_list",
                    "success": true,
                    "package_count": packages.len(),
                    "packages": packages,
                }));
                return Ok(());
            }
            let result = scoop::package_list_report(root).await;
            let lines = scoop::package_list_report_lines(root).await;
            logger::command_results(
                logger::CLI_SCOOP_PACKAGE_QUERY,
                std::slice::from_ref(&result),
            );
            output::print_lines(&lines);
            Ok(())
        }
        ScoopSubcommand::Search(ScoopSearchCommand { query }) => {
            let effective_root = effective_root(install_root);
            let Some(root) = effective_root.as_deref() else {
                print_cli_response(&messages::missing_scoop_root(), json_mode);
                return Ok(());
            };
            if scoop::load_buckets_from_registry(root).await.is_empty() {
                scoop::ensure_main_bucket_ready(root).await?;
            }
            if json_mode {
                output::print_json_value(&scoop::search_results(root, query.as_deref()).await);
                return Ok(());
            }
            let result = scoop::search_report(root, query.as_deref()).await;
            let lines = scoop::search_report_lines(root, query.as_deref()).await;
            logger::command_results(logger::CLI_SCOOP_SEARCH, std::slice::from_ref(&result));
            output::print_search_result_lines(&lines, query.as_deref());
            Ok(())
        }
        ScoopSubcommand::Info(ScoopSinglePackageCommand { package }) => {
            let effective_root = effective_root(install_root);
            let Some(root) = effective_root.as_deref() else {
                print_cli_response(&messages::missing_scoop_root(), json_mode);
                return Ok(());
            };
            if json_mode {
                output::print_json_value(&scoop::package_info(root, &package).await);
                return Ok(());
            }
            let result = scoop::package_info_report(root, &package).await;
            let lines = scoop::package_info_report_lines(root, &package).await;
            logger::command_results(
                logger::CLI_SCOOP_PACKAGE_QUERY,
                std::slice::from_ref(&result),
            );
            output::print_lines(&lines);
            Ok(())
        }
        ScoopSubcommand::Cat(ScoopSinglePackageCommand { package }) => {
            let effective_root = effective_root(install_root);
            let Some(root) = effective_root.as_deref() else {
                print_cli_response(&messages::missing_scoop_root(), json_mode);
                return Ok(());
            };
            let result = scoop::package_manifest(root, &package).await;
            let lines = scoop::package_manifest_lines(root, &package).await;
            logger::command_results(
                logger::CLI_SCOOP_PACKAGE_QUERY,
                std::slice::from_ref(&result),
            );
            if json_mode {
                output::print_json_lines(&lines);
            } else {
                output::print_lines(&lines);
            }
            Ok(())
        }
        ScoopSubcommand::Prefix(ScoopSinglePackageCommand { package }) => {
            let effective_root = effective_root(install_root);
            let Some(root) = effective_root.as_deref() else {
                print_cli_response(&messages::missing_scoop_root(), json_mode);
                return Ok(());
            };
            if json_mode {
                let layout = spoon_core::RuntimeLayout::from_root(root);
                let prefix = layout.scoop.apps_root.join(&package).join("current");
                let status_data = scoop::runtime_status(root).await;
                let installed_version = status_data
                    .installed_packages
                    .iter()
                    .find(|p| p.name == package)
                    .map(|p| p.version.trim().to_string());
                let installed = installed_version.is_some() && prefix.exists();
                output::print_json_value(&json!({
                    "kind": "package_prefix",
                    "success": installed,
                    "package": package,
                    "installed": installed,
                    "installed_version": installed_version,
                    "prefix": installed.then(|| prefix.display().to_string()),
                    "message": (!installed).then(|| format!("Scoop package '{package}' is not installed.")),
                }));
                return Ok(());
            }
            let result = scoop::package_prefix_report(root, &package).await;
            let lines = scoop::package_prefix_report_lines(root, &package).await;
            logger::command_results(
                logger::CLI_SCOOP_PACKAGE_QUERY,
                std::slice::from_ref(&result),
            );
            output::print_lines(&lines);
            Ok(())
        }
        ScoopSubcommand::Install(command) => {
            run_scoop_package_command("install", &command, install_root, json_mode)
        }
        ScoopSubcommand::Update(command) => {
            run_scoop_package_command("update", &command, install_root, json_mode)
        }
        ScoopSubcommand::Uninstall(command) => {
            run_scoop_package_command("uninstall", &command, install_root, json_mode)
        }
        ScoopSubcommand::Cache { command } => {
            let effective_root = effective_root(install_root);
            let Some(root) = effective_root.as_deref() else {
                print_cli_response(&messages::missing_scoop_root(), json_mode);
                return Ok(());
            };
            run_domain_cache_command(command, CacheScope::Scoop, root, json_mode);
            Ok(())
        }
        ScoopSubcommand::Bucket { command } => {
            let effective_root = effective_root(install_root);
            let Some(root) = effective_root.as_deref() else {
                print_cli_response(&messages::missing_scoop_root(), json_mode);
                return Ok(());
            };
            let result = match &command {
                ScoopBucketSubcommand::List => {
                    if json_mode {
                        output::print_json_value(&scoop::bucket_inventory(root).await);
                        return Ok(());
                    }
                    let result = scoop::bucket_list_report(root).await;
                    let lines = scoop::bucket_list_report_lines(root).await;
                    output::print_lines(&lines);
                    result
                }
                ScoopBucketSubcommand::Add(command) => {
                    let source = command
                        .source
                        .clone()
                        .or_else(|| {
                            scoop::known_bucket_source(&command.name)
                        })
                        .with_context(|| {
                            format!(
                                "bucket '{}' requires an explicit source; no well-known bucket mapping exists",
                                command.name
                            )
                        })?;
                    scoop::bucket_add(root, &command.name, &source, &command.branch).await?
                }
                ScoopBucketSubcommand::Update(command) => {
                    scoop::bucket_update(root, &command.names).await?
                }
                ScoopBucketSubcommand::Remove(command) => {
                    scoop::bucket_remove(root, &command.name).await?
                }
            };
            logger::command_results(
                logger::CLI_SCOOP_BUCKET_ACTION,
                std::slice::from_ref(&result),
            );
            if json_mode {
                let (action, targets) = match &command {
                    ScoopBucketSubcommand::List => ("list", Vec::new()),
                    ScoopBucketSubcommand::Add(command) => ("add", vec![command.name.clone()]),
                    ScoopBucketSubcommand::Update(command) => ("update", command.names.clone()),
                    ScoopBucketSubcommand::Remove(command) => {
                        ("remove", vec![command.name.clone()])
                    }
                };
                output::print_json_value(&scoop::bucket_action_result(
                    root, action, &targets, &result,
                ));
            } else {
                print_command_result(&result, false);
            }
            Ok(())
        }
    }
}

pub async fn run_command(
    command: Commands,
    install_root: Option<&Path>,
    cli_root: Option<PathBuf>,
    json_mode: bool,
) -> Result<()> {
    match command {
        Commands::Status(StatusCommand { refresh }) => {
            if refresh {
                if let Some(root) = effective_root(install_root) {
                    let result = scoop::bucket_update_with_emit(
                        &root,
                        &[],
                        if json_mode {
                            |_| {}
                        } else {
                            output::print_stream_chunk
                        },
                    )
                    .await?;
                    if json_mode {
                        output::print_json_value(&json!({
                            "kind": "status_refresh",
                            "bucket_update": scoop::bucket_action_result(
                                &root,
                                "update",
                                &Vec::<String>::new(),
                                &result,
                            ),
                            "status": cli_json::status_view(Some(&root), true),
                        }));
                    } else {
                        print_command_result(&result, false);
                        status::print_status(Some(&root), true);
                    }
                } else {
                    if json_mode {
                        output::print_json_value(&cli_json::status_view(install_root, true));
                    } else {
                        status::print_status(install_root, true);
                    }
                }
            } else {
                if json_mode {
                    output::print_json_value(&cli_json::status_view(install_root, false));
                } else {
                    status::print_status(install_root, false);
                }
            }
        }
        Commands::Doctor => {
            let effective_root = effective_root(install_root);
            let Some(root) = effective_root.as_deref() else {
                print_cli_response(&messages::missing_scoop_root(), json_mode);
                return Ok(());
            };
            if json_mode {
                output::print_json_value(&scoop::doctor_report(root).await?);
                return Ok(());
            }
            let result = scoop::doctor_summary(root).await?;
            let lines = scoop::doctor_summary_lines(root).await?;
            logger::command_results(logger::CLI_SCOOP_DOCTOR, std::slice::from_ref(&result));
            output::print_lines(&lines);
        }
        Commands::Install(command) => {
            run_scoop_package_command("install", &command, install_root, json_mode)?;
        }
        Commands::Update(command) => {
            run_scoop_package_command("update", &command, install_root, json_mode)?;
        }
        Commands::Uninstall(command) => {
            run_scoop_package_command("uninstall", &command, install_root, json_mode)?;
        }
        Commands::List => {
            run_scoop_command(ScoopSubcommand::List, install_root, json_mode).await?;
        }
        Commands::Search(command) => {
            run_scoop_command(ScoopSubcommand::Search(command), install_root, json_mode).await?;
        }
        Commands::Info(command) => {
            run_scoop_command(ScoopSubcommand::Info(command), install_root, json_mode).await?;
        }
        Commands::Cat(command) => {
            run_scoop_command(ScoopSubcommand::Cat(command), install_root, json_mode).await?;
        }
        Commands::Prefix(command) => {
            run_scoop_command(ScoopSubcommand::Prefix(command), install_root, json_mode).await?;
        }
        Commands::Bucket { command } => {
            run_scoop_command(ScoopSubcommand::Bucket { command }, install_root, json_mode).await?;
        }
        Commands::Scoop { command } => {
            run_scoop_command(command, install_root, json_mode).await?;
        }
        Commands::Msvc { command } => {
            run_msvc_command(command, install_root, json_mode).await?;
        }
        Commands::Config { command } => match command {
            None => print_config_view(json_mode),
            Some(ConfigSubcommand::Path) => print_config_path(json_mode),
            Some(ConfigSubcommand::Cat) => print_config_cat(json_mode)?,
            Some(ConfigSubcommand::Root(command)) => {
                run_root_config_command(command, cli_root, json_mode)?
            }
            Some(ConfigSubcommand::Msvc(command)) => {
                run_package_config_command(command, "msvc", json_mode).await?
            }
            Some(ConfigSubcommand::Python(command)) => {
                run_package_config_command(command, "python", json_mode).await?
            }
            Some(ConfigSubcommand::Git(command)) => {
                run_package_config_command(command, "git", json_mode).await?
            }
        },
    }

    Ok(())
}
