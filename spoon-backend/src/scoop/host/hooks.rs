use std::path::Path;

use minijinja::{Environment, context};
use strum_macros::AsRefStr;
use tokio::process::Command;

use crate::{BackendError, Result};
use crate::platform::msiexec_path;
use crate::scoop::current_architecture_key;

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum HookPhase {
    PreInstall,
    Installer,
    PostInstall,
    PreUninstall,
    Uninstaller,
    PostUninstall,
}

impl HookPhase {
    fn warning_only(self) -> bool {
        matches!(self, Self::PostUninstall)
    }
}

/// Inputs required to render and execute a Scoop lifecycle hook block.
///
/// This groups the hook's working roots, contextual package metadata, and any
/// optional helper executables that the PowerShell prelude exposes.
pub struct HookExecutionContext<'a> {
    /// Logical command surface (`install` / `uninstall`) exposed to the script.
    pub command_name: &'a str,
    /// Package version exposed to the script prelude.
    pub version: &'a str,
    /// Root where the package version/current asset content is being operated on.
    pub install_root: &'a Path,
    /// Persist root paired with the install root for this package.
    pub persist_root: &'a Path,
    /// Optional archive path used by installer/unpacker helpers.
    pub archive_path: Option<&'a Path>,
    /// Current package name exposed to the hook prelude as `$app`.
    pub app: Option<&'a str>,
    /// Current bucket name exposed to the hook prelude as `$bucket`.
    pub bucket: Option<&'a str>,
    /// Bucket registry root exposed to the hook prelude as `$bucketsdir`.
    pub buckets_dir: Option<&'a Path>,
    /// Optional dark helper path for Wix/Burn extraction commands.
    pub dark_helper_path: Option<&'a Path>,
    /// Optional innounp helper path for Inno Setup extraction commands.
    pub innounp_helper_path: Option<&'a Path>,
}

pub async fn execute_hook_scripts(
    scripts: &[String],
    phase: HookPhase,
    execution: &HookExecutionContext<'_>,
) -> Result<()> {
    if scripts.is_empty() {
        return Ok(());
    }
    let script_block = scripts.join("\n");
    tracing::info!(
        "Running {} hook script block ({} line(s)).",
        phase.as_ref(),
        scripts.len()
    );
    let archive_name = execution
        .archive_path
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let mut env = Environment::new();
    env.add_filter("ps", |value: String| value.replace('\'', "''"));
    env.add_template(
        "hook_prelude",
        include_str!("templates/hook_prelude.j2.ps1"),
    )
    .map_err(|err| BackendError::HookTemplate(format!("failed to load hook prelude template: {err}")))?;
    let template = env
        .get_template("hook_prelude")
        .map_err(|err| BackendError::HookTemplate(format!("failed to access hook prelude template: {err}")))?;
    let prelude = template
        .render(context! {
            msiexec_path => msiexec_path().display().to_string(),
            command_name => execution.command_name,
            install_root => execution.install_root.display().to_string(),
            persist_root => execution.persist_root.display().to_string(),
            archive_name => archive_name,
            version => execution.version,
            architecture => current_architecture_key(),
            context_app => execution.app.map(ToString::to_string),
            context_bucket => execution.bucket.map(ToString::to_string),
            context_buckets_dir => execution.buckets_dir.map(|path| path.display().to_string()),
            dark_helper_path => execution.dark_helper_path.map(|path| path.display().to_string()),
            innounp_helper_path => execution.innounp_helper_path.map(|path| path.display().to_string()),
        })
        .map_err(|err| BackendError::HookTemplate(format!("failed to render hook prelude template: {err}")))?;
    let rendered = format!("{prelude}\n{script_block}");
    let working_dir = if execution.install_root.exists() {
        execution.install_root
    } else if execution.persist_root.exists() {
        execution.persist_root
    } else {
        Path::new(".")
    };
    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(rendered)
        .current_dir(working_dir)
        .output()
        .await
        .map_err(|err| {
            BackendError::external("failed to execute Scoop lifecycle hook script", err)
        });
    let result = match output {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(BackendError::HookExecutionFailed {
                status: output.status.code(),
                stdout,
                stderr,
            })
        }
        Err(error) => Err(error),
    };
    if let Err(error) = result {
        if phase.warning_only() {
            tracing::warn!("Hook warning (ignored during {}): {}", phase.as_ref(), error);
        } else {
            return Err(error);
        }
    }
    Ok(())
}
