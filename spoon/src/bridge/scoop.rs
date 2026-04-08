use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context as _, Result as AnyResult};

use super::{CommandResult, CommandStatus, ConfigEntry, StreamChunk, stream_chunk_from_event};
pub(crate) use super::{desired_policy_entries, APP_PORTS};
use super::CancellationToken;

pub use spoon_scoop::ensure_main_bucket_ready;
pub use spoon_scoop::{
    latest_version, latest_version_async, load_manifest,
    load_manifest_sync, load_package_manifest, load_package_manifest_sync, parse_manifest,
    resolve_manifest, upsert_bucket_to_registry,
};
pub use spoon_scoop::{BucketSpec, known_bucket_source};

pub(crate) use spoon_scoop::{
    ScoopBucketOperationOutcome,
    add_bucket as add_bucket_to_registry_outcome,
    remove_bucket as remove_bucket_from_registry_outcome,
    update_buckets as update_buckets_outcome,
};
pub(crate) use spoon_scoop::ScoopDoctorDetails;
pub(crate) use spoon_scoop::{
    ScoopActionPackage, ScoopBucketInventory, ScoopPackageActionOutcome,
    ScoopPackageInstallState, ScoopPackageOperationOutcome, ScoopPackagePlan,
};
pub(crate) use spoon_scoop::load_buckets_from_registry;
pub(crate) use spoon_scoop::{
    infer_tool_root_with_overrides as infer_tool_root,
    plan_package_action_with_display as plan_package_action,
};
pub(crate) use spoon_scoop::{installed_package_states, runtime_status, search_results};

pub type ScoopPackageDetailsOutcome = spoon_scoop::ScoopPackageDetailsOutcome<ConfigEntry>;

static REAL_BACKEND_TEST_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_real_backend_test_mode(enabled: bool) {
    REAL_BACKEND_TEST_MODE.store(enabled, Ordering::Relaxed);
}

pub(crate) fn should_fake() -> bool {
    super::test_mode_enabled() && !REAL_BACKEND_TEST_MODE.load(Ordering::Relaxed)
}

pub(crate) fn configured_proxy() -> String {
    crate::config::load_global_config().proxy.clone()
}

pub(crate) fn command_result(
    title: impl Into<String>,
    status: CommandStatus,
) -> CommandResult {
    CommandResult {
        title: title.into(),
        status,
    }
}

pub(crate) fn command_result_from_scoop_package_outcome(
    outcome: ScoopPackageOperationOutcome,
) -> CommandResult {
    command_result(outcome.title, outcome.status)
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum RunMode {
    Install,
    Update,
    Uninstall,
}

pub async fn package_info(tool_root: &Path, package_name: &str) -> ScoopPackageDetailsOutcome {
    let desired_policy = desired_policy_entries(package_name);
    let mut outcome = spoon_scoop::package_info::<ConfigEntry>(tool_root, package_name).await;
    if let ScoopPackageDetailsOutcome::Details(details) = &mut outcome {
        details.integration.policy.desired = desired_policy;
    }
    outcome
}

// ---------------------------------------------------------------------------
// Runtime bridge (was service/scoop/runtime.rs)
// ---------------------------------------------------------------------------

pub(crate) async fn doctor_details(
    tool_root: &Path,
) -> AnyResult<spoon_scoop::ScoopDoctorDetails> {
    Ok(spoon_scoop::doctor(tool_root).await)
}

pub(crate) fn resolved_pip_mirror_url_for_display(policy_value: &str) -> String {
    super::resolved_pip_mirror_url_for_display(policy_value)
}

pub(crate) async fn reapply_package_integrations(
    tool_root: &Path,
    package_name: &str,
) -> AnyResult<Vec<String>> {
    let layout = spoon_core::RuntimeLayout::from_root(tool_root);
    spoon_scoop::reapply_integrations(&layout.scoop, package_name, &APP_PORTS, None)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    Ok(vec![format!(
        "Reapplied integrations for '{}'.",
        package_name
    )])
}

pub(crate) async fn reapply_package_integrations_with_emit<F>(
    tool_root: &Path,
    package_name: &str,
    mut emit: F,
) -> AnyResult<Vec<String>>
where
    F: FnMut(StreamChunk),
{
    let layout = spoon_core::RuntimeLayout::from_root(tool_root);
    let (sender, mut receiver) = spoon_core::event_bus(64);

    spoon_scoop::reapply_integrations(&layout.scoop, package_name, &APP_PORTS, Some(&sender))
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    while let Ok(Some(event)) = receiver.try_recv() {
        if let Some(chunk) = stream_chunk_from_event(event) {
            emit(chunk);
        }
    }

    Ok(vec![format!(
        "Reapplied integrations for '{}'.",
        package_name
    )])
}

pub(crate) async fn reapply_package_command_surface(
    tool_root: &Path,
    package_name: &str,
) -> AnyResult<Vec<String>> {
    let layout = spoon_core::RuntimeLayout::from_root(tool_root);
    spoon_scoop::reapply_command_surface(
        &layout.scoop,
        &layout.shims,
        package_name,
        &APP_PORTS,
        None,
    )
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    Ok(vec![format!(
        "Reapplied command surface for '{}'.",
        package_name
    )])
}

pub(crate) async fn reapply_package_command_surface_with_emit<F>(
    tool_root: &Path,
    package_name: &str,
    mut emit: F,
) -> AnyResult<Vec<String>>
where
    F: FnMut(StreamChunk),
{
    let layout = spoon_core::RuntimeLayout::from_root(tool_root);
    let (sender, mut receiver) = spoon_core::event_bus(64);

    spoon_scoop::reapply_command_surface(
        &layout.scoop,
        &layout.shims,
        package_name,
        &APP_PORTS,
        Some(&sender),
    )
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    while let Ok(Some(event)) = receiver.try_recv() {
        if let Some(chunk) = stream_chunk_from_event(event) {
            emit(chunk);
        }
    }

    Ok(vec![format!(
        "Reapplied command surface for '{}'.",
        package_name
    )])
}

/// Execute a package install/update/uninstall action using spoon-scoop.
///
/// Events are forwarded to the caller's emit closure after the operation completes.
pub(crate) async fn execute_package_action_outcome_streaming(
    tool_root: &Path,
    plan: &ScoopPackagePlan,
    proxy: &str,
    cancel: Option<&CancellationToken>,
    emit: Option<&mut dyn FnMut(StreamChunk)>,
) -> AnyResult<ScoopPackageOperationOutcome> {
    let layout = spoon_core::RuntimeLayout::from_root(tool_root);
    let scoop_layout = &layout.scoop;

    // Create event bus for collecting operation events
    let (sender, mut receiver) = spoon_core::event_bus(64);

    // Build HTTP client with proxy support
    let client = spoon_core::ReqwestClientBuilder::new()
        .proxy(proxy)
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .build()
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Execute the action via spoon-scoop
    let result: spoon_scoop::Result<()> = match plan.action {
        spoon_scoop::ScoopPackageAction::Install | spoon_scoop::ScoopPackageAction::Update => {
            spoon_scoop::install_package(
                scoop_layout,
                &client,
                plan,
                proxy,
                cancel,
                &APP_PORTS,
                Some(&sender),
            )
            .await
        }
        spoon_scoop::ScoopPackageAction::Uninstall => {
            spoon_scoop::uninstall_package(scoop_layout, plan, &APP_PORTS, Some(&sender)).await
        }
        spoon_scoop::ScoopPackageAction::Reapply => {
            spoon_scoop::uninstall_package(scoop_layout, plan, &APP_PORTS, Some(&sender)).await
        }
        _ => Err(spoon_scoop::ScoopError::Other(format!(
            "unsupported action: {:?}",
            plan.action
        ))),
    };

    // Forward collected events to the caller
    if let Some(emit) = emit {
        while let Ok(Some(event)) = receiver.try_recv() {
            if let Some(chunk) = stream_chunk_from_event(event) {
                emit(chunk);
            }
        }
    }

    // Build outcome
    let status = if result.is_ok() {
        spoon_core::CommandStatus::Success
    } else {
        spoon_core::CommandStatus::Failed
    };

    Ok(ScoopPackageOperationOutcome {
        kind: "package_operation",
        action: plan.action.as_str().to_string(),
        package: spoon_scoop::ScoopActionPackage {
            name: plan.package_name.clone(),
            display_name: plan.display_name.clone(),
        },
        status,
        title: plan.title(),
        state: Default::default(),
    })
}

// ---------------------------------------------------------------------------
// Actions (was service/scoop/actions.rs)
// ---------------------------------------------------------------------------

use super::PackageRef;

fn configured_root_override() -> Option<String> {
    let trimmed = crate::config::load_global_config().root.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn fake_result(
    action: &str,
    display_name: &str,
    package_name: &str,
    tool_root: Option<&Path>,
) -> CommandResult {
    let initial_plan = plan_package_action(action, display_name, package_name, tool_root);
    let mut output = Vec::new();
    if let Some(line) = initial_plan.resolution_line() {
        output.push(line);
    }
    output.push(initial_plan.command_line());
    output.push(format!(
        "Test mode: skipped real Scoop {action} for {display_name}."
    ));
    command_result(
        initial_plan.title(),
        CommandStatus::Success,
    )
}

fn run_scoop(action: &str, pkg: PackageRef) -> AnyResult<CommandResult> {
    run_scoop_streaming(action, pkg, None, Option::<fn(StreamChunk)>::None)
}

pub(crate) fn run_scoop_streaming<F>(
    action: &str,
    pkg: PackageRef,
    cancel: Option<&CancellationToken>,
    mut emit: Option<F>,
) -> AnyResult<CommandResult>
where
    F: FnMut(StreamChunk),
{
    let root_override = configured_root_override();
    let tool_root = infer_tool_root(None, root_override.as_deref());

    if should_fake() {
        let result = fake_result(
            action,
            pkg.display_name,
            pkg.package_name,
            tool_root.as_deref(),
        );
        if let Some(ref mut emit) = emit {
            let initial_plan = plan_package_action(action, pkg.display_name, pkg.package_name, tool_root.as_deref());
            if let Some(line) = initial_plan.resolution_line() {
                emit(StreamChunk::Append(line));
            }
            emit(StreamChunk::Append(initial_plan.command_line()));
            emit(StreamChunk::Append(format!(
                "Test mode: skipped real Scoop {action} for {}.",
                pkg.display_name
            )));
        }
        return Ok(result);
    }

    let configured_tool_root =
        tool_root.context("Scoop package actions require a configured root")?;
    let plan = plan_package_action(
        action,
        pkg.display_name,
        pkg.package_name,
        Some(&configured_tool_root),
    );
    let mut emit_dyn = emit
        .as_mut()
        .map(|emit| emit as &mut dyn FnMut(StreamChunk));
    let outcome = crate::runtime::block_on_sync(execute_package_action_outcome_streaming(
        &configured_tool_root,
        &plan,
        &configured_proxy(),
        cancel,
        emit_dyn.as_deref_mut(),
    ))?;
    Ok(command_result_from_scoop_package_outcome(outcome))
}

pub(crate) fn run_package_action_streaming<F>(
    action: &str,
    display_name: &str,
    package_name: &str,
    install_root: Option<&Path>,
    cancel: Option<&CancellationToken>,
    mut emit: Option<F>,
) -> AnyResult<CommandResult>
where
    F: FnMut(StreamChunk),
{
    let root_override = configured_root_override();
    let root_for_plan = infer_tool_root(install_root, root_override.as_deref());

    if should_fake() {
        let result = fake_result(
            action,
            display_name,
            package_name,
            root_for_plan.as_deref(),
        );
        if let Some(ref mut emit) = emit {
            let initial_plan = plan_package_action(action, display_name, package_name, root_for_plan.as_deref());
            if let Some(line) = initial_plan.resolution_line() {
                emit(StreamChunk::Append(line));
            }
            emit(StreamChunk::Append(initial_plan.command_line()));
            emit(StreamChunk::Append(format!(
                "Test mode: skipped real Scoop {action} for {display_name}."
            )));
        }
        return Ok(result);
    }

    let tool_root = root_for_plan.context("Scoop package actions require a configured root")?;
    let plan = plan_package_action(action, display_name, package_name, Some(&tool_root));
    let mut emit_dyn = emit
        .as_mut()
        .map(|emit| emit as &mut dyn FnMut(StreamChunk));
    let outcome = crate::runtime::block_on_sync(execute_package_action_outcome_streaming(
        &tool_root,
        &plan,
        &configured_proxy(),
        cancel,
        emit_dyn.as_deref_mut(),
    ))?;
    Ok(command_result_from_scoop_package_outcome(outcome))
}

fn run_many(
    mode: RunMode,
    packages: &[PackageRef],
    _install_root: Option<&Path>,
) -> AnyResult<Vec<CommandResult>> {
    let mut results = Vec::new();

    for pkg in packages {
        let action = match mode {
            RunMode::Install => run_scoop("install", *pkg),
            RunMode::Update => run_scoop("update", *pkg),
            RunMode::Uninstall => run_scoop("uninstall", *pkg),
        }?;
        results.push(action);
    }

    Ok(results)
}

pub fn install_tools(
    packages: &[PackageRef],
    install_root: Option<&Path>,
) -> AnyResult<Vec<CommandResult>> {
    run_many(RunMode::Install, packages, install_root)
}

pub fn update_tools(
    packages: &[PackageRef],
    install_root: Option<&Path>,
) -> AnyResult<Vec<CommandResult>> {
    run_many(RunMode::Update, packages, install_root)
}

pub fn uninstall_tools(
    packages: &[PackageRef],
    install_root: Option<&Path>,
) -> AnyResult<Vec<CommandResult>> {
    run_many(RunMode::Uninstall, packages, install_root)
}

pub(crate) fn install_tools_streaming<F>(
    packages: &[PackageRef],
    install_root: Option<&Path>,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> AnyResult<Vec<CommandResult>>
where
    F: FnMut(StreamChunk),
{
    run_many_streaming(RunMode::Install, packages, install_root, cancel, emit)
}

pub(crate) fn update_tools_streaming<F>(
    packages: &[PackageRef],
    install_root: Option<&Path>,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> AnyResult<Vec<CommandResult>>
where
    F: FnMut(StreamChunk),
{
    run_many_streaming(RunMode::Update, packages, install_root, cancel, emit)
}

pub(crate) fn uninstall_tools_streaming<F>(
    packages: &[PackageRef],
    install_root: Option<&Path>,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> AnyResult<Vec<CommandResult>>
where
    F: FnMut(StreamChunk),
{
    run_many_streaming(RunMode::Uninstall, packages, install_root, cancel, emit)
}

fn run_many_streaming<F>(
    mode: RunMode,
    packages: &[PackageRef],
    _install_root: Option<&Path>,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> AnyResult<Vec<CommandResult>>
where
    F: FnMut(StreamChunk),
{
    let mut results = Vec::new();

    for pkg in packages {
        let action = match mode {
            RunMode::Install => run_scoop_streaming("install", *pkg, cancel, Some(&mut *emit)),
            RunMode::Update => run_scoop_streaming("update", *pkg, cancel, Some(&mut *emit)),
            RunMode::Uninstall => run_scoop_streaming("uninstall", *pkg, cancel, Some(&mut *emit)),
        }?;
        results.push(action);
    }

    Ok(results)
}

pub fn package_action_result(
    tool_root: &Path,
    action: &str,
    package_name: &str,
    display_name: &str,
    result: &CommandResult,
) -> AnyResult<ScoopPackageActionOutcome> {
    let installed_state = crate::runtime::block_on_sync(spoon_scoop::read_installed_state(
        &spoon_core::RuntimeLayout::from_root(tool_root).scoop,
        package_name,
    ))
    .ok()
    .flatten();
    let (installed, installed_version, current) = match installed_state {
        Some(state) => {
            let version = state.version().to_string();
            let layout = spoon_core::RuntimeLayout::from_root(tool_root);
            let current_path = layout.scoop.package_current_root(package_name);
            (true, Some(version), Some(current_path.display().to_string()))
        }
        None => (false, None, None),
    };
    Ok(ScoopPackageActionOutcome {
        kind: "scoop_package_action",
        action: action.to_string(),
        package: ScoopActionPackage {
            name: package_name.to_string(),
            display_name: display_name.to_string(),
        },
        success: result.is_success(),
        title: result.title.clone(),
        state: ScoopPackageInstallState {
            installed,
            installed_version,
            current,
        },
    })
}

// ---------------------------------------------------------------------------
// Bucket operations (was service/scoop/bucket.rs)
// ---------------------------------------------------------------------------

pub use spoon_core::RepoSyncOutcome;

fn command_result_from_bucket_outcome(outcome: ScoopBucketOperationOutcome) -> CommandResult {
    command_result(outcome.title, outcome.status)
}

pub async fn bucket_list_report(tool_root: &Path) -> CommandResult {
    let _output = bucket_list_report_lines(tool_root).await;
    command_result("list Scoop buckets", CommandStatus::Success)
}

pub async fn bucket_list_report_lines(tool_root: &Path) -> Vec<String> {
    let buckets = load_buckets_from_registry(tool_root).await;
    let mut output = Vec::new();
    if buckets.is_empty() {
        output.push("No Scoop buckets are registered.".to_string());
    } else {
        for bucket in buckets {
            output.push(format!(
                "{} | {} | {}",
                bucket.name, bucket.branch, bucket.source
            ));
        }
    }
    output
}

pub async fn bucket_inventory(tool_root: &Path) -> ScoopBucketInventory {
    let buckets = load_buckets_from_registry(tool_root).await;
    ScoopBucketInventory {
        kind: "scoop_bucket_list",
        success: true,
        bucket_count: buckets.len(),
        buckets,
    }
}

pub async fn doctor_summary(tool_root: &Path) -> AnyResult<CommandResult> {
    let _details = doctor_report(tool_root).await?;
    Ok(command_result(
        "doctor Scoop runtime",
        CommandStatus::Success,
    ))
}

pub async fn doctor_summary_lines(tool_root: &Path) -> AnyResult<Vec<String>> {
    let details = doctor_report(tool_root).await?;
    let mut output = details
        .ensured_paths
        .into_iter()
        .map(|path| format!("Ensured Scoop directory: {path}"))
        .collect::<Vec<_>>();
    output.push(format!(
        "Registered Scoop buckets: {}",
        details.registered_buckets.len()
    ));
    output.push(format!("Scoop state root: {}", details.runtime.state_root));
    Ok(output)
}

pub async fn doctor_report(tool_root: &Path) -> AnyResult<ScoopDoctorDetails> {
    doctor_details(tool_root).await
}

pub fn bucket_action_result(
    tool_root: &Path,
    action: &str,
    target_names: &[String],
    result: &CommandResult,
) -> ScoopBucketOperationOutcome {
    let buckets = crate::runtime::block_on_sync(load_buckets_from_registry(tool_root));
    ScoopBucketOperationOutcome {
        kind: "scoop_bucket_action",
        action: action.to_string(),
        targets: target_names.to_vec(),
        status: result.status,
        title: result.title.clone(),
        bucket_count: buckets.len(),
        buckets,
    }
}

pub async fn bucket_add(
    tool_root: &Path,
    name: &str,
    source: &str,
    branch: &str,
) -> AnyResult<CommandResult> {
    let spec = BucketSpec {
        name: name.to_string(),
        source: Some(source.to_string()),
        branch: Some(branch.to_string()),
    };
    Ok(
        add_bucket_to_registry_outcome(tool_root, &spec, &configured_proxy())
            .await
            .map(command_result_from_bucket_outcome)?,
    )
}

pub async fn bucket_remove(tool_root: &Path, name: &str) -> AnyResult<CommandResult> {
    Ok(remove_bucket_from_registry_outcome(tool_root, name)
        .await
        .map(command_result_from_bucket_outcome)?)
}

pub async fn bucket_update(tool_root: &Path, names: &[String]) -> AnyResult<CommandResult> {
    Ok(
        update_buckets_outcome(tool_root, names, &configured_proxy())
            .await
            .map(command_result_from_bucket_outcome)?,
    )
}

/// Run bucket update with FnMut(StreamChunk) forwarding for CLI callers.
pub async fn bucket_update_with_emit<F>(
    tool_root: &Path,
    names: &[String],
    mut emit: F,
) -> AnyResult<CommandResult>
where
    F: FnMut(StreamChunk),
{
    let (_sender, mut receiver) = spoon_core::event_bus(64);
    let result = update_buckets_outcome(tool_root, names, &configured_proxy())
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Forward collected events
    while let Ok(Some(event)) = receiver.try_recv() {
        if let Some(chunk) = stream_chunk_from_event(event) {
            emit(chunk);
        }
    }

    Ok(command_result_from_bucket_outcome(result))
}

// ---------------------------------------------------------------------------
// Report functions (was service/scoop/report.rs)
// ---------------------------------------------------------------------------

use super::format_bytes;

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
    let _ = package_list_report_lines(tool_root).await;
    command_result("list Scoop packages", CommandStatus::Success)
}

pub async fn package_list_report_lines(tool_root: &Path) -> Vec<String> {
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
    lines_or_default(
        packages,
        "No Scoop packages are currently installed.",
        |package| format!("{} | {}", package.name, package.version),
    )
}

pub async fn package_prefix_report(tool_root: &Path, package_name: &str) -> CommandResult {
    let layout = spoon_core::RuntimeLayout::from_root(tool_root);
    let prefix = layout.scoop.apps_root.join(package_name).join("current");
    let status_data = runtime_status(tool_root).await;
    let installed_version = status_data
        .installed_packages
        .iter()
        .find(|p| p.name == package_name)
        .map(|p| p.version.trim().to_string());
    let installed = installed_version.is_some() && prefix.exists();
    let status = if installed {
        CommandStatus::Success
    } else {
        CommandStatus::Failed
    };
    command_result(
        format!("prefix Scoop package {package_name}"),
        status,
    )
}

pub async fn package_prefix_report_lines(tool_root: &Path, package_name: &str) -> Vec<String> {
    let layout = spoon_core::RuntimeLayout::from_root(tool_root);
    let prefix = layout.scoop.apps_root.join(package_name).join("current");
    let status_data = runtime_status(tool_root).await;
    let installed_version = status_data
        .installed_packages
        .iter()
        .find(|p| p.name == package_name)
        .map(|p| p.version.trim().to_string());
    let installed = installed_version.is_some() && prefix.exists();
    if installed {
        vec![prefix.display().to_string()]
    } else {
        vec![format!("Scoop package '{package_name}' is not installed.")]
    }
}

pub async fn runtime_status_report(_tool_root: &Path) -> CommandResult {
    command_result(
        "status Scoop runtime",
        CommandStatus::Success,
    )
}

pub async fn runtime_status_report_lines(tool_root: &Path) -> Vec<String> {
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
    output
}

pub async fn search_report(tool_root: &Path, query: Option<&str>) -> CommandResult {
    let data = search_results(tool_root, query).await;
    let title = match data.query {
        Some(query) => format!("search Scoop packages for {query}"),
        None => "search Scoop packages".to_string(),
    };
    command_result(title, CommandStatus::Success)
}

pub async fn search_report_lines(tool_root: &Path, query: Option<&str>) -> Vec<String> {
    let data = search_results(tool_root, query).await;
    lines_or_default(data.matches, "No matching Scoop packages found.", |item| {
        format!(
            "{} | {} | {} | {}",
            item.package_name,
            item.version.unwrap_or_else(|| "-".to_string()),
            item.bucket,
            item.description.unwrap_or_default()
        )
    })
}

pub async fn package_info_report(tool_root: &Path, package_name: &str) -> CommandResult {
    match package_info(tool_root, package_name).await {
        ScoopPackageDetailsOutcome::Details(_) => {
            command_result(
                format!("info Scoop package {package_name}"),
                CommandStatus::Success,
            )
        }
        ScoopPackageDetailsOutcome::Error(error) => command_result(
            format!("info Scoop package {}", error.package),
            CommandStatus::Failed,
        ),
    }
}

pub async fn package_info_report_lines(tool_root: &Path, package_name: &str) -> Vec<String> {
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
            output
        }
        ScoopPackageDetailsOutcome::Error(error) => vec![error.error.message],
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
    command_result(outcome.title, outcome.status)
}

pub async fn package_manifest_lines(tool_root: &Path, package_name: &str) -> Vec<String> {
    let outcome = spoon_scoop::package_manifest(tool_root, package_name).await;
    match (outcome.content, outcome.error) {
        (Some(content), _) => content.lines().map(str::to_string).collect(),
        (None, Some(error)) => vec![error.message],
        (None, None) => Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Tests (was service/scoop/actions.rs tests)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::config;
    use crate::bridge::{PackageRef, StreamChunk};

    use super::{run_package_action_streaming, run_scoop_streaming};

    #[test]
    fn fake_streaming_install_produces_success_result() {
        config::enable_test_mode();
        let pkg = PackageRef {
            display_name: "uv",
            package_name: "uv",
        };
        let result = run_scoop_streaming("install", pkg, None, Option::<fn(StreamChunk)>::None)
            .expect("fake scoop install");
        assert!(result.is_success());
    }

    #[test]
    fn fake_package_install_produces_success_result() {
        config::enable_test_mode();
        let result = run_package_action_streaming(
            "install",
            "uv",
            "uv",
            None,
            None,
            Option::<fn(StreamChunk)>::None,
        )
        .expect("fake package install");
        assert!(result.is_success());
    }
}
