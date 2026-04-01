use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use fs_err as fs;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::BackendContext;
use crate::{
    BackendError, BackendEvent, CancellationToken, CommandStatus, ProgressEvent, Result,
    check_token_cancel, event::progress_kind,
};

use super::common::{http_client, push_stream_line, unique_existing_dirs};
use super::paths;
use super::rules::pick_higher_version;
use super::validation::{
    RustValidationTemplateOptions, locate_cargo, write_validation_cpp_template,
    write_validation_rust_templates,
};
use super::{
    MsvcCanonicalState, MsvcLifecycleStage, MsvcOperationKind, MsvcRuntimeKind,
    MsvcValidationStatus, OfficialMsvcStateDetail, read_canonical_state, write_canonical_state,
};

const OFFICIAL_BUILD_TOOLS_BOOTSTRAPPER_URL: &str =
    "https://aka.ms/vs/17/release/vs_BuildTools.exe";
const OFFICIAL_BUILD_TOOLS_WORKLOADS: &[&str] = &[
    "Microsoft.VisualStudio.Workload.VCTools",
    "Microsoft.VisualStudio.Workload.NativeDesktop",
];
const OFFICIAL_BUILD_TOOLS_COMPONENTS: &[&str] = &[
    "Microsoft.VisualStudio.ComponentGroup.NativeDesktop.Core",
    "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
    "Microsoft.VisualStudio.Component.Windows10SDK",
    "Microsoft.VisualStudio.Component.Windows11SDK.26100",
    "Microsoft.Component.VC.Runtime.UCRTSDK",
];

fn external<T, E>(result: std::result::Result<T, E>, message: impl Into<String>) -> Result<T>
where
    E: std::error::Error + Send + Sync + 'static,
{
    result.map_err(|err| BackendError::external(message.into(), err))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfficialInstallerMode {
    Quiet,
    Passive,
}

impl OfficialInstallerMode {
    fn as_cli_token(self) -> &'static str {
        match self {
            Self::Quiet => "quiet",
            Self::Passive => "passive",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficialInstalledState {
    pub version: Option<String>,
    pub sdk_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OfficialRuntimeState {
    runtime: String,
    instance_root: String,
    bootstrapper_path: String,
    last_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OfficialCommandMetadata {
    action: String,
    bootstrapper_source: String,
    bootstrapper_path: String,
    args: Vec<String>,
    log_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OfficialAction {
    Install,
    Update,
    Uninstall,
}

impl OfficialAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::Install => "install",
            Self::Update => "update",
            Self::Uninstall => "uninstall",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Install => "install MSVC Toolchain",
            Self::Update => "update MSVC Toolchain",
            Self::Uninstall => "uninstall MSVC Toolchain",
        }
    }
}

pub fn runtime_state_path(tool_root: &Path) -> PathBuf {
    paths::official_msvc_state_root(tool_root).join("runtime.json")
}

pub fn installed_state_path(tool_root: &Path) -> PathBuf {
    paths::official_msvc_state_root(tool_root).join("installed.json")
}

fn command_metadata_path(tool_root: &Path) -> PathBuf {
    paths::official_msvc_cache_root(tool_root)
        .join("commands")
        .join("last-command.json")
}

fn bootstrapper_dir(tool_root: &Path) -> PathBuf {
    paths::official_msvc_cache_root(tool_root).join("bootstrapper")
}

fn logs_dir(tool_root: &Path) -> PathBuf {
    paths::official_msvc_cache_root(tool_root).join("logs")
}

pub fn windows_kits_root() -> PathBuf {
    PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10")
}

pub fn vswhere_path() -> PathBuf {
    PathBuf::from(r"C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe")
}

fn managed_windows_kits_root(tool_root: &Path) -> PathBuf {
    paths::msvc_toolchain_root(tool_root)
        .join("Windows Kits")
        .join("10")
}

pub fn official_instance_root(tool_root: &Path) -> PathBuf {
    paths::official_msvc_root(tool_root)
}

pub fn read_installed_version_label(tool_root: &Path) -> Option<String> {
    let content = fs::read_to_string(installed_state_path(tool_root)).ok()?;
    let state = serde_json::from_str::<OfficialInstalledState>(&content).ok()?;
    match (state.version, state.sdk_version) {
        (Some(version), Some(sdk)) => Some(format!("{version} + {sdk}")),
        (Some(version), None) => Some(version),
        (None, Some(sdk)) => Some(format!("SDK {sdk}")),
        (None, None) => None,
    }
}

pub fn probe(tool_root: &Path) -> (PathBuf, bool, Option<String>) {
    let root = official_instance_root(tool_root);
    let runtime_state = runtime_state_path(tool_root);
    let installed = read_installed_version_label(tool_root);
    let available = runtime_state.exists();
    (root, available, installed)
}

pub async fn install_toolchain_async_with_mode(
    tool_root: &Path,
    mode: OfficialInstallerMode,
) -> Result<super::MsvcOperationOutcome> {
    let request = super::MsvcRequest::for_tool_root(tool_root);
    run_official_action_async(&request, OfficialAction::Install, mode, None, None).await
}

pub async fn update_toolchain_async_with_mode(
    tool_root: &Path,
    mode: OfficialInstallerMode,
) -> Result<super::MsvcOperationOutcome> {
    let request = super::MsvcRequest::for_tool_root(tool_root);
    run_official_action_async(&request, OfficialAction::Update, mode, None, None).await
}

pub async fn uninstall_toolchain_async(
    tool_root: &Path,
    mode: OfficialInstallerMode,
) -> Result<super::MsvcOperationOutcome> {
    let request = super::MsvcRequest::for_tool_root(tool_root);
    run_official_action_async(&request, OfficialAction::Uninstall, mode, None, None).await
}

pub async fn install_toolchain_async_with_mode_and_context<P>(
    context: &BackendContext<P>,
    mode: OfficialInstallerMode,
) -> Result<super::MsvcOperationOutcome> {
    let request = super::MsvcRequest::from_context(context);
    run_official_action_async(&request, OfficialAction::Install, mode, None, None).await
}

pub async fn update_toolchain_async_with_mode_and_context<P>(
    context: &BackendContext<P>,
    mode: OfficialInstallerMode,
) -> Result<super::MsvcOperationOutcome> {
    let request = super::MsvcRequest::from_context(context);
    run_official_action_async(&request, OfficialAction::Update, mode, None, None).await
}

pub async fn uninstall_toolchain_async_with_context<P>(
    context: &BackendContext<P>,
    mode: OfficialInstallerMode,
) -> Result<super::MsvcOperationOutcome> {
    let request = super::MsvcRequest::from_context(context);
    run_official_action_async(&request, OfficialAction::Uninstall, mode, None, None).await
}

pub async fn install_toolchain_streaming<F>(
    tool_root: &Path,
    mode: OfficialInstallerMode,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> Result<super::MsvcOperationOutcome>
where
    F: FnMut(BackendEvent),
{
    let request = super::MsvcRequest::for_tool_root(tool_root);
    let mut callback = emit as &mut dyn FnMut(BackendEvent);
    let mut result = run_official_action_async(
        &request,
        OfficialAction::Install,
        mode,
        cancel,
        Some(&mut callback),
    )
    .await?;
    result.streamed = true;
    Ok(result)
}

pub async fn update_toolchain_streaming<F>(
    tool_root: &Path,
    mode: OfficialInstallerMode,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> Result<super::MsvcOperationOutcome>
where
    F: FnMut(BackendEvent),
{
    let request = super::MsvcRequest::for_tool_root(tool_root);
    let mut callback = emit as &mut dyn FnMut(BackendEvent);
    let mut result = run_official_action_async(
        &request,
        OfficialAction::Update,
        mode,
        cancel,
        Some(&mut callback),
    )
    .await?;
    result.streamed = true;
    Ok(result)
}

pub async fn uninstall_toolchain_streaming<F>(
    tool_root: &Path,
    mode: OfficialInstallerMode,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> Result<super::MsvcOperationOutcome>
where
    F: FnMut(BackendEvent),
{
    let request = super::MsvcRequest::for_tool_root(tool_root);
    let mut callback = emit as &mut dyn FnMut(BackendEvent);
    let mut result = run_official_action_async(
        &request,
        OfficialAction::Uninstall,
        mode,
        cancel,
        Some(&mut callback),
    )
    .await?;
    result.streamed = true;
    Ok(result)
}

pub async fn install_toolchain_streaming_with_context<P, F>(
    context: &BackendContext<P>,
    mode: OfficialInstallerMode,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> Result<super::MsvcOperationOutcome>
where
    F: FnMut(BackendEvent),
{
    let request = super::MsvcRequest::from_context(context);
    let mut callback = emit as &mut dyn FnMut(BackendEvent);
    let mut result = run_official_action_async(
        &request,
        OfficialAction::Install,
        mode,
        cancel,
        Some(&mut callback),
    )
    .await?;
    result.streamed = true;
    Ok(result)
}

pub async fn update_toolchain_streaming_with_context<P, F>(
    context: &BackendContext<P>,
    mode: OfficialInstallerMode,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> Result<super::MsvcOperationOutcome>
where
    F: FnMut(BackendEvent),
{
    let request = super::MsvcRequest::from_context(context);
    let mut callback = emit as &mut dyn FnMut(BackendEvent);
    let mut result = run_official_action_async(
        &request,
        OfficialAction::Update,
        mode,
        cancel,
        Some(&mut callback),
    )
    .await?;
    result.streamed = true;
    Ok(result)
}

pub async fn uninstall_toolchain_streaming_with_context<P, F>(
    context: &BackendContext<P>,
    mode: OfficialInstallerMode,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> Result<super::MsvcOperationOutcome>
where
    F: FnMut(BackendEvent),
{
    let request = super::MsvcRequest::from_context(context);
    let mut callback = emit as &mut dyn FnMut(BackendEvent);
    let mut result = run_official_action_async(
        &request,
        OfficialAction::Uninstall,
        mode,
        cancel,
        Some(&mut callback),
    )
    .await?;
    result.streamed = true;
    Ok(result)
}

fn bootstrapper_source() -> String {
    std::env::var("SPOON_TEST_MSVCOFFICIAL_BOOTSTRAPPER_SOURCE")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| OFFICIAL_BUILD_TOOLS_BOOTSTRAPPER_URL.to_string())
}

fn bootstrapper_cache_path(tool_root: &Path, source: &str) -> PathBuf {
    let file_name = Path::new(source)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .or_else(|| {
            source
                .rsplit('/')
                .next()
                .filter(|name| !name.trim().is_empty())
        })
        .unwrap_or("vs_BuildTools.exe");
    bootstrapper_dir(tool_root).join(file_name)
}

async fn cache_bootstrapper(
    tool_root: &Path,
    proxy: &str,
    cancel: Option<&CancellationToken>,
    emit: &mut Option<&mut dyn FnMut(BackendEvent)>,
) -> Result<(String, PathBuf, Vec<String>)> {
    let source = bootstrapper_source();
    let destination = bootstrapper_cache_path(tool_root, &source);
    external(
        fs::create_dir_all(bootstrapper_dir(tool_root)),
        format!("failed to create {}", bootstrapper_dir(tool_root).display()),
    )?;

    let mut lines = Vec::new();
    if destination.exists() {
        push_stream_line(
            &mut lines,
            emit,
            format!(
                "Reused cached official MSVC bootstrapper at {}",
                destination.display()
            ),
        );
        return Ok((source, destination, lines));
    }

    push_stream_line(
        &mut lines,
        emit,
        format!("Caching official MSVC bootstrapper from {}", source),
    );

    if let Some(local) = source.strip_prefix("file:///") {
        external(
            fs::copy(local, &destination),
            format!(
                "failed to copy official MSVC bootstrapper from {} to {}",
                local,
                destination.display()
            ),
        )?;
    } else if Path::new(&source).exists() {
        external(
            fs::copy(&source, &destination),
            format!(
                "failed to copy official MSVC bootstrapper from {} to {}",
                source,
                destination.display()
            ),
        )?;
    } else {
        let client = http_client(proxy)?;
        check_token_cancel(cancel)?;
        let mut response = client
            .get(&source)
            .send()
            .await
            .map_err(|err| BackendError::network(&source, err))?
            .error_for_status()
            .map_err(|err| BackendError::network(&source, err))?;
        let total_bytes = response.content_length();
        let mut downloaded_bytes = 0_u64;
        let target_label = destination
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("vs_BuildTools.exe");
        if let Some(total_bytes) = total_bytes {
            if let Some(callback) = emit.as_deref_mut() {
                callback(BackendEvent::Progress(ProgressEvent::bytes(
                    progress_kind::DOWNLOAD,
                    target_label,
                    0,
                    Some(total_bytes),
                )));
            }
        } else if let Some(callback) = emit.as_deref_mut() {
            callback(BackendEvent::Progress(ProgressEvent::bytes(
                progress_kind::DOWNLOAD,
                target_label,
                0,
                None,
            )));
        }
        let mut file = external(
            fs::File::create(&destination),
            format!("failed to create {}", destination.display()),
        )?;
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|err| BackendError::network(&source, err))?
        {
            check_token_cancel(cancel)?;
            use std::io::Write as _;
            external(
                file.write_all(&chunk),
                format!("failed to write {}", destination.display()),
            )?;
            downloaded_bytes += chunk.len() as u64;
            if let Some(total_bytes) = total_bytes {
                if let Some(callback) = emit.as_deref_mut() {
                    callback(BackendEvent::Progress(ProgressEvent::bytes(
                        progress_kind::DOWNLOAD,
                        target_label,
                        downloaded_bytes,
                        Some(total_bytes),
                    )));
                }
            } else if let Some(callback) = emit.as_deref_mut() {
                callback(BackendEvent::Progress(ProgressEvent::bytes(
                    progress_kind::DOWNLOAD,
                    target_label,
                    downloaded_bytes,
                    None,
                )));
            }
        }
    }

    push_stream_line(
        &mut lines,
        emit,
        format!(
            "Cached official MSVC bootstrapper at {}",
            destination.display()
        ),
    );
    Ok((source, destination, lines))
}

fn official_action_args(
    tool_root: &Path,
    action: OfficialAction,
    mode: OfficialInstallerMode,
) -> (PathBuf, Vec<String>) {
    let log_path = logs_dir(tool_root).join("vs_buildtools_install.log");
    let mut args = Vec::new();
    if matches!(action, OfficialAction::Uninstall) {
        args.push("uninstall".to_string());
    }
    args.push(format!("--{}", mode.as_cli_token()));
    args.push("--wait".to_string());
    args.push("--norestart".to_string());
    args.push("--nocache".to_string());
    args.push("--installPath".to_string());
    args.push(official_instance_root(tool_root).display().to_string());
    if !matches!(action, OfficialAction::Uninstall) {
        args.push("--includeRecommended".to_string());
        for workload in OFFICIAL_BUILD_TOOLS_WORKLOADS {
            args.push("--add".to_string());
            args.push((*workload).to_string());
        }
        for component in OFFICIAL_BUILD_TOOLS_COMPONENTS {
            args.push("--add".to_string());
            args.push((*component).to_string());
        }
    }
    (log_path.clone(), args)
}

fn write_command_metadata(
    tool_root: &Path,
    action: OfficialAction,
    bootstrapper_source: &str,
    bootstrapper_path: &Path,
    log_path: &Path,
    args: &[String],
) -> Result<()> {
    let path = command_metadata_path(tool_root);
    if let Some(parent) = path.parent() {
        external(
            fs::create_dir_all(parent),
            format!("failed to create {}", parent.display()),
        )?;
    }
    let content = serde_json::to_string_pretty(&OfficialCommandMetadata {
        action: action.as_str().to_string(),
        bootstrapper_source: bootstrapper_source.to_string(),
        bootstrapper_path: bootstrapper_path.display().to_string(),
        args: args.to_vec(),
        log_path: log_path.display().to_string(),
    })?;
    external(
        fs::write(&path, content),
        format!("failed to write {}", path.display()),
    )?;
    Ok(())
}

fn detect_msvc_version(instance_root: &Path) -> Option<String> {
    let vc_root = instance_root.join("VC").join("Tools").join("MSVC");
    let entries = fs::read_dir(vc_root).ok()?;
    let mut best: Option<String> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let version = entry.file_name().to_string_lossy().to_string();
        pick_higher_version(&mut best, format!("msvc-{version}"));
    }
    best.map(|version| version.trim_start_matches("msvc-").to_string())
}

fn detect_sdk_version(instance_root: &Path) -> Option<String> {
    let instance_sdk_root = instance_root.join("Windows Kits").join("10").join("Lib");
    let sdk_root = if instance_sdk_root.exists() {
        instance_sdk_root
    } else {
        windows_kits_root().join("Lib")
    };
    let entries = fs::read_dir(sdk_root).ok()?;
    let mut best: Option<String> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let version = entry.file_name().to_string_lossy().to_string();
        pick_higher_version(&mut best, format!("sdk-{version}"));
    }
    best.map(|version| version.trim_start_matches("sdk-").to_string())
}

fn find_preferred_binary(root: &Path, candidates: &[&str], target_arch: &str) -> Option<PathBuf> {
    let host_arch = paths::native_msvc_arch().to_ascii_lowercase();
    let mut matches = WalkDir::new(root)
        .into_iter()
        .flatten()
        .filter(|entry| {
            entry.file_type().is_file()
                && candidates.iter().any(|candidate| {
                    entry
                        .file_name()
                        .to_str()
                        .is_some_and(|name| name.eq_ignore_ascii_case(candidate))
                })
        })
        .map(|entry| entry.into_path())
        .collect::<Vec<_>>();
    matches.sort_by_key(|path| {
        let lowered = path.display().to_string().to_ascii_lowercase();
        let host_target = format!("host{}\\{}", host_arch, target_arch);
        let host_native = format!("host{}\\", host_arch);
        (
            !lowered.contains(&host_target),
            !lowered.contains(&host_native),
            lowered,
        )
    });
    matches.into_iter().next()
}

fn sdk_include_dirs_from_root(root: &Path) -> Vec<PathBuf> {
    let sdk_include_root = root.join("Include");
    let mut dirs = Vec::new();
    if let Ok(entries) = fs::read_dir(&sdk_include_root) {
        for entry in entries.flatten() {
            let version_root = entry.path();
            if !version_root.is_dir() {
                continue;
            }
            for segment in ["ucrt", "shared", "um", "winrt", "cppwinrt"] {
                let dir = version_root.join(segment);
                if dir.is_dir() {
                    dirs.push(dir);
                }
            }
        }
    }
    unique_existing_dirs(dirs)
}

fn sdk_lib_dirs_from_root(root: &Path, target_arch: &str) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for candidate in ["kernel32.lib", "ucrt.lib", "libcmt.lib", "user32.lib"] {
        for entry in WalkDir::new(root.join("Lib")).into_iter().flatten() {
            if entry.file_type().is_file()
                && entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.eq_ignore_ascii_case(candidate))
            {
                if let Some(parent) = entry.path().parent() {
                    let parent = parent.to_path_buf();
                    let file_name_matches = parent
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.eq_ignore_ascii_case(target_arch));
                    if file_name_matches {
                        dirs.push(parent);
                    }
                }
            }
        }
    }
    unique_existing_dirs(dirs)
}

fn official_include_dirs(instance_root: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let vc_tools_root = instance_root.join("VC").join("Tools").join("MSVC");
    if let Ok(entries) = fs::read_dir(&vc_tools_root) {
        for entry in entries.flatten() {
            let include = entry.path().join("include");
            if include.is_dir() {
                dirs.push(include);
            }
        }
    }

    let instance_sdk_root = instance_root.join("Windows Kits").join("10");
    let sdk_root = if instance_sdk_root.exists() {
        instance_sdk_root
    } else {
        windows_kits_root()
    };
    dirs.extend(sdk_include_dirs_from_root(&sdk_root));
    unique_existing_dirs(dirs)
}

fn official_lib_dirs(instance_root: &Path, target_arch: &str) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let vc_root = instance_root.to_path_buf();
    let sdk_root = {
        let instance_sdk_root = instance_root.join("Windows Kits").join("10");
        if instance_sdk_root.exists() {
            instance_sdk_root
        } else {
            windows_kits_root()
        }
    };
    dirs.extend(sdk_lib_dirs_from_root(&sdk_root, target_arch));
    for candidate in ["kernel32.lib", "ucrt.lib", "libcmt.lib", "user32.lib"] {
        for entry in WalkDir::new(&vc_root).into_iter().flatten() {
            if entry.file_type().is_file()
                && entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.eq_ignore_ascii_case(candidate))
            {
                if let Some(parent) = entry.path().parent() {
                    let parent = parent.to_path_buf();
                    let file_name_matches = parent
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.eq_ignore_ascii_case(&target_arch));
                    if file_name_matches {
                        dirs.push(parent);
                    }
                }
            }
        }
    }
    unique_existing_dirs(dirs)
}

fn has_desktop_windows_sdk(include_dirs: &[PathBuf], lib_dirs: &[PathBuf]) -> bool {
    let has_windows_h = include_dirs
        .iter()
        .any(|dir| dir.join("Windows.h").exists());
    let has_kernel32 = lib_dirs.iter().any(|dir| dir.join("kernel32.lib").exists());
    has_windows_h && has_kernel32
}

fn validate_toolchain_layout(tool_root: &Path) -> Result<(PathBuf, Vec<String>)> {
    let instance_root = official_instance_root(tool_root);
    let runtime_state = runtime_state_path(tool_root);
    let installed_state = installed_state_path(tool_root);
    let mut lines = vec![format!(
        "Inspecting official toolchain under {}",
        instance_root.display()
    )];
    if !instance_root.exists() {
        return Err(BackendError::Other(format!(
            "official MSVC instance root does not exist: {}",
            instance_root.display()
        )));
    }
    if !runtime_state.exists() {
        return Err(BackendError::Other(format!(
            "official MSVC runtime state is missing: {}",
            runtime_state.display()
        )));
    }
    if !installed_state.exists() {
        return Err(BackendError::Other(format!(
            "official MSVC installed state is missing: {}",
            installed_state.display()
        )));
    }
    lines.push(format!(
        "Found official runtime state at {}",
        runtime_state.display()
    ));
    lines.push(format!(
        "Found official installed state at {}",
        installed_state.display()
    ));
    Ok((instance_root, lines))
}

fn write_validation_workspace_scripts(
    cpp_root: &Path,
    source: &Path,
    output: &Path,
    compiler: &Path,
    include_dirs: &[PathBuf],
    lib_dirs: &[PathBuf],
    rust_root: &Path,
    cargo_path: Option<&Path>,
) -> Result<()> {
    let build_script = cpp_root.join("build.cmd");
    let rust_build_script = rust_root.join("build.cmd");
    let include_flags = include_dirs
        .iter()
        .map(|path| format!("/I\"{}\"", path.display()))
        .collect::<Vec<_>>()
        .join(" ");
    let lib_flags = lib_dirs
        .iter()
        .map(|path| format!("/LIBPATH:\"{}\"", path.display()))
        .collect::<Vec<_>>()
        .join(" ");
    let script = format!(
        "@echo off\r\n\
setlocal\r\n\
\"{}\" /nologo /std:c++17 {} \"{}\" /Fe:\"{}\" /link {} user32.lib\r\n",
        compiler.display(),
        include_flags,
        source.display(),
        output.display(),
        lib_flags
    );
    fs::write(&build_script, script)
        .map_err(|err| BackendError::fs("write", &build_script, err))?;
    let cargo = cargo_path
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "cargo".to_string());
    fs::write(
        &rust_build_script,
        format!(
            concat!(
                "@echo off\r\n",
                "setlocal\r\n",
                "rem validate rust build goes through Cargo + build.rs + official cl.exe\r\n",
                "set \"SPOON_VALIDATE_SPOON_CL={compiler}\"\r\n",
                "\"{cargo}\" build --quiet\r\n",
            ),
            cargo = cargo,
            compiler = compiler.display(),
        ),
    )
    .map_err(|err| {
        BackendError::Other(format!(
            "failed to write {}: {err}",
            rust_build_script.display()
        ))
    })?;
    Ok(())
}

pub async fn validate_toolchain(tool_root: &Path) -> Result<super::MsvcOperationOutcome> {
    let request = super::MsvcRequest::for_tool_root(tool_root);
    validate_toolchain_with_request(&request).await
}

pub async fn validate_toolchain_with_context<P>(
    context: &BackendContext<P>,
) -> Result<super::MsvcOperationOutcome> {
    let request = super::MsvcRequest::from_context(context);
    validate_toolchain_with_request(&request).await
}

async fn validate_toolchain_with_request(
    request: &super::MsvcRequest,
) -> Result<super::MsvcOperationOutcome> {
    let tool_root = request.root.as_path();
    let (instance_root, mut lines) = validate_toolchain_layout(tool_root)?;
    let target_arch = request.normalized_target_arch();
    let compiler = find_preferred_binary(
        &instance_root,
        &["cl.exe", "cl.cmd", "cl.bat"],
        &target_arch,
    )
    .ok_or_else(|| {
        BackendError::Other(format!(
            "official MSVC compiler was not found under {}",
            instance_root.display()
        ))
    })?;
    let linker = find_preferred_binary(
        &instance_root,
        &["link.exe", "link.cmd", "link.bat"],
        &target_arch,
    )
    .ok_or_else(|| {
        BackendError::Other(format!(
            "official MSVC linker was not found under {}",
            instance_root.display()
        ))
    })?;
    let mut include_dirs = official_include_dirs(&instance_root);
    let target_arch = request.normalized_target_arch();
    let mut lib_dirs = official_lib_dirs(&instance_root, &target_arch);
    if !has_desktop_windows_sdk(&include_dirs, &lib_dirs) {
        let managed_sdk_root = managed_windows_kits_root(tool_root);
        if managed_sdk_root.exists() {
            include_dirs.extend(sdk_include_dirs_from_root(&managed_sdk_root));
            lib_dirs.extend(sdk_lib_dirs_from_root(&managed_sdk_root, &target_arch));
            include_dirs = unique_existing_dirs(include_dirs);
            lib_dirs = unique_existing_dirs(lib_dirs);
            lines.push(format!(
                "Official Windows SDK layout is incomplete; falling back to managed SDK content under {}",
                managed_sdk_root.display()
            ));
        }
    }
    if include_dirs.is_empty() {
        return Err(BackendError::Other(format!(
            "no official include directories were discovered under {}",
            instance_root.display()
        )));
    }
    if lib_dirs.is_empty() {
        return Err(BackendError::Other(format!(
            "no official library directories were discovered under {}",
            instance_root.display()
        )));
    }

    lines.push(format!("Using official compiler at {}", compiler.display()));
    lines.push(format!("Using official linker at {}", linker.display()));
    lines.push(format!(
        "Discovered {} official include directories.",
        include_dirs.len()
    ));
    lines.push(format!(
        "Discovered {} official library directories.",
        lib_dirs.len()
    ));

    let validate_root = paths::official_msvc_cache_root(tool_root).join("validate");
    if validate_root.exists() {
        let _ = fs::remove_dir_all(&validate_root);
    }
    external(
        fs::create_dir_all(&validate_root),
        format!("failed to create {}", validate_root.display()),
    )?;
    lines.push(format!(
        "Prepared official validation workspace {}.",
        validate_root.display()
    ));

    let cpp_root = validate_root.join("cpp");
    let rust_root = validate_root.join("rust");
    external(
        fs::create_dir_all(&cpp_root),
        format!("failed to create {}", cpp_root.display()),
    )?;
    external(
        fs::create_dir_all(&rust_root),
        format!("failed to create {}", rust_root.display()),
    )?;
    let source = write_validation_cpp_template(&cpp_root)?;
    let output = cpp_root.join("hello.exe");
    write_validation_rust_templates(
        &rust_root,
        RustValidationTemplateOptions {
            linker: &linker,
            sample_label: "spoon msvc validate official rust",
            native_helper_label: "official-cl",
            linker_label: "official-link",
        },
    )?;
    let rust_output = rust_root
        .join("target")
        .join("debug")
        .join("hello-rust.exe");
    let cargo = locate_cargo();
    write_validation_workspace_scripts(
        &cpp_root,
        &source,
        &output,
        &compiler,
        &include_dirs,
        &lib_dirs,
        &rust_root,
        cargo.as_deref(),
    )?;

    let build_script = cpp_root.join("build.cmd");
    let compile_output = Command::new("cmd.exe")
        .arg("/C")
        .arg(&build_script)
        .current_dir(&cpp_root)
        .output()
        .map_err(|err| {
            BackendError::external(format!("failed to run {}", build_script.display()), err)
        })?;
    if !compile_output.status.success() {
        return Err(BackendError::Other(format!(
            "official validation compile failed\ninclude_dirs:\n{}\nlib_dirs:\n{}\nstdout:\n{}\nstderr:\n{}",
            include_dirs
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join("\n"),
            lib_dirs
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join("\n"),
            String::from_utf8_lossy(&compile_output.stdout).trim(),
            String::from_utf8_lossy(&compile_output.stderr).trim()
        )));
    }
    lines.push("C++ validation:".to_string());
    lines.push("  Compiled official C++/Win32 validation sample successfully.".to_string());

    let run_output = Command::new(&output)
        .current_dir(&cpp_root)
        .output()
        .map_err(|err| {
            BackendError::external(format!("failed to run {}", output.display()), err)
        })?;
    if !run_output.status.success() {
        return Err(BackendError::Other(format!(
            "official validation sample failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&run_output.stdout).trim(),
            String::from_utf8_lossy(&run_output.stderr).trim()
        )));
    }
    let sample_stdout_lines = String::from_utf8_lossy(&run_output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    lines.extend(
        sample_stdout_lines
            .into_iter()
            .map(|line| format!("  {line}")),
    );
    lines.push("  Ran official validation sample successfully.".to_string());
    lines.push(format!(
        "  Compiled official C++/Win32 validation sample successfully into {}.",
        output.display()
    ));
    match cargo {
        Some(cargo_path) => {
            lines.push("Rust validation:".to_string());
            lines.push(format!("  Using Cargo at {}", cargo_path.display()));
            if request.test_mode {
                lines.push(
                    "  Skipped official Rust validation execution in test mode; generated Cargo sample and build script for inspection."
                        .to_string(),
                );
            } else {
                let rust_compile_output = {
                    let mut command = Command::new(&cargo_path);
                    command
                        .current_dir(&rust_root)
                        .env("SPOON_VALIDATE_SPOON_CL", &compiler)
                        .arg("build")
                        .arg("--quiet");
                    command.output().map_err(|err| {
                        BackendError::external(
                            format!("failed to run {}", cargo_path.display()),
                            err,
                        )
                    })?
                };
                let rust_stdout_lines = String::from_utf8_lossy(&rust_compile_output.stdout)
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                let rust_stderr_lines = String::from_utf8_lossy(&rust_compile_output.stderr)
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                lines.push(
                    "  Compiled official Rust/Cargo validation sample successfully.".to_string(),
                );
                lines.extend(
                    rust_stdout_lines
                        .into_iter()
                        .map(|line| format!("  {line}")),
                );
                lines.extend(
                    rust_stderr_lines
                        .into_iter()
                        .map(|line| format!("  {line}")),
                );
                if !rust_compile_output.status.success() {
                    let details = if lines.is_empty() {
                        "no rust compiler output captured".to_string()
                    } else {
                        lines.join("\n")
                    };
                    return Err(BackendError::Other(format!(
                        "official MSVC validation cargo sample failed in {}\n{}",
                        rust_root.display(),
                        details
                    )));
                }
                if !rust_output.exists() {
                    return Err(BackendError::Other(format!(
                        "official MSVC validation did not produce {}",
                        rust_output.display()
                    )));
                }
                let rust_run_output = Command::new(&rust_output)
                    .current_dir(&rust_root)
                    .output()
                    .map_err(|err| {
                        BackendError::external(
                            format!("failed to run {}", rust_output.display()),
                            err,
                        )
                    })?;
                let rust_run_stdout_lines = String::from_utf8_lossy(&rust_run_output.stdout)
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                let rust_run_stderr_lines = String::from_utf8_lossy(&rust_run_output.stderr)
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                lines.extend(
                    rust_run_stdout_lines
                        .into_iter()
                        .map(|line| format!("  {line}")),
                );
                lines.extend(
                    rust_run_stderr_lines
                        .into_iter()
                        .map(|line| format!("  {line}")),
                );
                if !rust_run_output.status.success() {
                    let details = if lines.is_empty() {
                        "no rust runtime output captured".to_string()
                    } else {
                        lines.join("\n")
                    };
                    return Err(BackendError::Other(format!(
                        "official MSVC validation rust sample exited with status {:?}\n{}",
                        rust_run_output.status.code(),
                        details
                    )));
                }
                lines.push(format!(
                    "  Ran official Rust validation sample successfully from {}.",
                    rust_output.display()
                ));
                lines.push(format!(
                    "  Compiled official Rust/Cargo validation sample successfully into {}.",
                    rust_output.display()
                ));
            }
        }
        None => {
            lines.push("Rust validation:".to_string());
            lines.push(
                "  Skipped official Rust validation sample because cargo is not available on PATH."
                    .to_string(),
            );
        }
    }
    lines.push(format!(
        "Kept official validation workspace at {} for inspection.",
        validate_root.display()
    ));

    Ok(super::MsvcOperationOutcome {
        kind: "msvc_operation",
        runtime: super::MsvcRuntimeKind::Official,
        operation: super::MsvcOperationKind::Validate,
        title: "validate MSVC Toolchain".to_string(),
        status: CommandStatus::Success,
        output: lines,
        streamed: false,
    })
}

fn detect_installed_state(instance_root: &Path) -> OfficialInstalledState {
    OfficialInstalledState {
        version: detect_msvc_version(instance_root),
        sdk_version: detect_sdk_version(instance_root),
    }
}

fn write_runtime_state(
    tool_root: &Path,
    action: OfficialAction,
    bootstrapper_path: &Path,
) -> Result<()> {
    let path = runtime_state_path(tool_root);
    if let Some(parent) = path.parent() {
        external(
            fs::create_dir_all(parent),
            format!("failed to create {}", parent.display()),
        )?;
    }
    let content = serde_json::to_string_pretty(&OfficialRuntimeState {
        runtime: "official".to_string(),
        instance_root: official_instance_root(tool_root).display().to_string(),
        bootstrapper_path: bootstrapper_path.display().to_string(),
        last_action: action.as_str().to_string(),
    })?;
    external(
        fs::write(&path, content),
        format!("failed to write {}", path.display()),
    )?;
    Ok(())
}

fn write_installed_state(tool_root: &Path, state: &OfficialInstalledState) -> Result<()> {
    let path = installed_state_path(tool_root);
    if let Some(parent) = path.parent() {
        external(
            fs::create_dir_all(parent),
            format!("failed to create {}", parent.display()),
        )?;
    }
    let content = serde_json::to_string_pretty(state)?;
    external(
        fs::write(&path, content),
        format!("failed to write {}", path.display()),
    )?;
    Ok(())
}

async fn write_official_canonical_state(
    tool_root: &Path,
    operation: MsvcOperationKind,
    installed: bool,
    installer_mode: Option<OfficialInstallerMode>,
    validation_status: Option<MsvcValidationStatus>,
    validation_message: Option<String>,
) -> Result<()> {
    let layout = crate::layout::RuntimeLayout::from_root(tool_root);
    let previous = read_canonical_state(&layout).await;
    let detected = if installed {
        detect_installed_state(&official_instance_root(tool_root))
    } else {
        OfficialInstalledState {
            version: None,
            sdk_version: None,
        }
    };
    let state = MsvcCanonicalState {
        runtime_kind: MsvcRuntimeKind::Official,
        installed,
        version: detected.version,
        sdk_version: detected.sdk_version,
        last_operation: Some(operation),
        last_stage: Some(MsvcLifecycleStage::Completed),
        validation_status: validation_status.or_else(|| {
            previous
                .as_ref()
                .and_then(|state| state.validation_status.clone())
        }),
        validation_message: validation_message
            .or_else(|| previous.as_ref().and_then(|state| state.validation_message.clone())),
        managed: previous.as_ref().map(|state| state.managed.clone()).unwrap_or_default(),
        official: OfficialMsvcStateDetail {
            installer_mode: installer_mode.map(OfficialInstallerMode::as_cli_token).map(str::to_string),
        },
    };
    write_canonical_state(&layout, &state).await
}

fn run_bootstrapper(
    bootstrapper_path: &Path,
    args: &[String],
    tool_root: &Path,
    mode: OfficialInstallerMode,
    _emit: &mut Option<&mut dyn FnMut(BackendEvent)>,
) -> Result<()> {
    let extension = bootstrapper_path
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let mut command = if matches!(extension.as_str(), "cmd" | "bat") {
        let mut command = Command::new("cmd.exe");
        command.arg("/C").arg(bootstrapper_path);
        command
    } else {
        Command::new(bootstrapper_path)
    };
    let mut child = command
        .args(args)
        .env(
            "SPOON_OFFICIAL_INSTANCE_ROOT",
            official_instance_root(tool_root).display().to_string(),
        )
        .env(
            "SPOON_OFFICIAL_CACHE_ROOT",
            paths::official_msvc_cache_root(tool_root)
                .display()
                .to_string(),
        )
        .spawn()
        .map_err(|err| {
            BackendError::external(
                format!("failed to run {}", bootstrapper_path.display()),
                err,
            )
        })?;
    let start = Instant::now();
    let mut last_heartbeat = 0_u64;
    loop {
        if child
            .try_wait()
            .map_err(|err| {
                BackendError::external(
                    format!("failed to poll {}", bootstrapper_path.display()),
                    err,
                )
            })?
            .is_some()
        {
            break;
        }
        let elapsed = start.elapsed().as_secs();
        if matches!(mode, OfficialInstallerMode::Quiet)
            && elapsed >= 10
            && elapsed / 10 > last_heartbeat
        {
            last_heartbeat = elapsed / 10;
            tracing::info!(
                "Official Build Tools bootstrapper is still running... ({}s elapsed)",
                elapsed
            );
        }
        thread::sleep(Duration::from_secs(1));
    }
    let output = child.wait_with_output().map_err(|err| {
        BackendError::external(
            format!("failed to wait for {}", bootstrapper_path.display()),
            err,
        )
    })?;
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BackendError::Other(format!(
            "official MSVC bootstrapper failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            stdout.trim(),
            stderr.trim()
        )));
    }
    Ok(())
}

async fn run_official_action_async(
    request: &super::MsvcRequest,
    action: OfficialAction,
    mode: OfficialInstallerMode,
    cancel: Option<&CancellationToken>,
    mut emit: Option<&mut dyn FnMut(BackendEvent)>,
) -> Result<super::MsvcOperationOutcome> {
    let tool_root = request.root.as_path();
    let proxy = request.proxy.as_str();
    external(
        fs::create_dir_all(paths::official_msvc_cache_root(tool_root)),
        format!(
            "failed to create {}",
            paths::official_msvc_cache_root(tool_root).display()
        ),
    )?;
    external(
        fs::create_dir_all(logs_dir(tool_root)),
        format!("failed to create {}", logs_dir(tool_root).display()),
    )?;

    let mut lines = vec![format!(
        "Preparing official MSVC {} under {}",
        action.as_str(),
        official_instance_root(tool_root).display()
    )];
    tracing::info!("{}", lines[0]);
    push_stream_line(
        &mut lines,
        &mut emit,
        format!("Official installer mode: {}", mode.as_cli_token()),
    );

    let (instance_root, runtime_state, installed_state) = (
        official_instance_root(tool_root),
        runtime_state_path(tool_root),
        installed_state_path(tool_root),
    );
    if matches!(action, OfficialAction::Uninstall)
        && !instance_root.exists()
        && !runtime_state.exists()
        && !installed_state.exists()
    {
        push_stream_line(
            &mut lines,
            &mut emit,
            "Official MSVC runtime is not present; nothing to uninstall.".to_string(),
        );
        push_stream_line(
            &mut lines,
            &mut emit,
            format!(
                "Official MSVC cache is retained at {}",
                paths::official_msvc_cache_root(tool_root).display()
            ),
        );
        write_official_canonical_state(
            tool_root,
            super::MsvcOperationKind::Uninstall,
            false,
            Some(mode),
            None,
            None,
        )
        .await?;
        return Ok(super::MsvcOperationOutcome {
            kind: "msvc_operation",
            runtime: super::MsvcRuntimeKind::Official,
            operation: super::MsvcOperationKind::Uninstall,
            title: action.title().to_string(),
            status: CommandStatus::Success,
            output: lines,
            streamed: false,
        });
    }

    let (bootstrapper_source, bootstrapper_path, bootstrapper_lines) =
        cache_bootstrapper(tool_root, proxy, cancel, &mut emit).await?;
    lines.extend(bootstrapper_lines);

    let (log_path, args) = official_action_args(tool_root, action, mode);
    write_command_metadata(
        tool_root,
        action,
        &bootstrapper_source,
        &bootstrapper_path,
        &log_path,
        &args,
    )?;
    push_stream_line(
        &mut lines,
        &mut emit,
        format!(
            "Recorded official MSVC command metadata at {}",
            command_metadata_path(tool_root).display()
        ),
    );
    push_stream_line(
        &mut lines,
        &mut emit,
        format!(
            "Launching official Build Tools bootstrapper at {}",
            bootstrapper_path.display()
        ),
    );
    if matches!(mode, OfficialInstallerMode::Passive) {
        push_stream_line(
            &mut lines,
            &mut emit,
            "Showing official installer UI in passive mode; follow Microsoft setup for detailed progress."
                .to_string(),
        );
    }
    run_bootstrapper(&bootstrapper_path, &args, tool_root, mode, &mut emit)?;

    if matches!(action, OfficialAction::Uninstall) {
        if instance_root.exists() {
            return Err(BackendError::Other(format!(
                "official MSVC bootstrapper reported success but the instance root still exists: {}",
                instance_root.display()
            )));
        }
        if runtime_state.exists() {
            external(
                fs::remove_file(&runtime_state),
                format!("failed to remove {}", runtime_state.display()),
            )?;
        }
        if installed_state.exists() {
            external(
                fs::remove_file(&installed_state),
                format!("failed to remove {}", installed_state.display()),
            )?;
        }
        let state_root = paths::official_msvc_state_root(tool_root);
        if state_root.exists() {
            external(
                fs::remove_dir_all(&state_root),
                format!("failed to remove {}", state_root.display()),
            )?;
        }
        push_stream_line(
            &mut lines,
            &mut emit,
            "Uninstalled official MSVC runtime through the Microsoft bootstrapper.".to_string(),
        );
        push_stream_line(
            &mut lines,
            &mut emit,
            format!(
                "Official MSVC cache is retained at {}",
                paths::official_msvc_cache_root(tool_root).display()
            ),
        );
        write_official_canonical_state(
            tool_root,
            super::MsvcOperationKind::Uninstall,
            false,
            Some(mode),
            None,
            None,
        )
        .await?;
        return Ok(super::MsvcOperationOutcome {
            kind: "msvc_operation",
            runtime: super::MsvcRuntimeKind::Official,
            operation: super::MsvcOperationKind::Uninstall,
            title: action.title().to_string(),
            status: CommandStatus::Success,
            output: lines,
            streamed: false,
        });
    }

    let installed = detect_installed_state(&official_instance_root(tool_root));
    write_runtime_state(tool_root, action, &bootstrapper_path)?;
    write_installed_state(tool_root, &installed)?;
    push_stream_line(
        &mut lines,
        &mut emit,
        format!(
            "Installed official MSVC runtime into {}",
            official_instance_root(tool_root).display()
        ),
    );
    if let Some(version) = installed.version.as_deref() {
        push_stream_line(
            &mut lines,
            &mut emit,
            format!("Detected official MSVC version: {version}"),
        );
    }
    if let Some(sdk_version) = installed.sdk_version.as_deref() {
        push_stream_line(
            &mut lines,
            &mut emit,
            format!("Detected official Windows SDK version: {sdk_version}"),
        );
    }
    write_official_canonical_state(
        tool_root,
        match action {
            OfficialAction::Install => super::MsvcOperationKind::Install,
            OfficialAction::Update => super::MsvcOperationKind::Update,
            OfficialAction::Uninstall => super::MsvcOperationKind::Uninstall,
        },
        true,
        Some(mode),
        None,
        None,
    )
    .await?;
    Ok(super::MsvcOperationOutcome {
        kind: "msvc_operation",
        runtime: super::MsvcRuntimeKind::Official,
        operation: match action {
            OfficialAction::Install => super::MsvcOperationKind::Install,
            OfficialAction::Update => super::MsvcOperationKind::Update,
            OfficialAction::Uninstall => super::MsvcOperationKind::Uninstall,
        },
        title: action.title().to_string(),
        status: CommandStatus::Success,
        output: lines,
        streamed: false,
    })
}
