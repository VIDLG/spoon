use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{BackendError, BackendEvent, Result};
use crate::platform::msiexec_path;

use super::selected_architecture_key;

pub struct HookContext {
    pub app: String,
    pub bucket: Option<String>,
    pub buckets_dir: PathBuf,
}

fn render_hook_prelude(
    install_root: &Path,
    persist_root: &Path,
    archive_path: Option<&Path>,
    context: Option<&HookContext>,
    command_name: &str,
    version: &str,
    dark_helper_path: Option<&Path>,
    innounp_helper_path: Option<&Path>,
) -> String {
    let archive_name = archive_path
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let mut prefix = String::new();
    prefix.push_str("$ErrorActionPreference='Continue'; ");
    prefix.push_str("if (Get-Variable -Name PSNativeCommandUseErrorActionPreference -ErrorAction SilentlyContinue) { $PSNativeCommandUseErrorActionPreference = $false }; ");
    prefix.push_str("function ensure([string]$p) { if (-not (Test-Path -LiteralPath $p)) { New-Item -ItemType Directory -Path $p -Force | Out-Null }; Resolve-Path -LiteralPath $p }; ");
    prefix.push_str(
        "function info([string]$m) { Write-Host \"INFO  $m\" -ForegroundColor DarkGray }; ",
    );
    prefix.push_str(
        "function warn([string]$m) { Write-Host \"WARN  $m\" -ForegroundColor DarkYellow }; ",
    );
    prefix.push_str(
        "function error([string]$m) { Write-Host \"ERROR $m\" -ForegroundColor DarkRed }; ",
    );
    prefix.push_str("function abort([string]$m) { throw $m }; ");
    prefix.push_str("function movedir([string]$from,[string]$to) { ");
    prefix.push_str("$from = $from.TrimEnd('\\\\'); $to = $to.TrimEnd('\\\\'); ");
    prefix.push_str("$proc = New-Object System.Diagnostics.Process; ");
    prefix.push_str("$proc.StartInfo.FileName = 'robocopy.exe'; ");
    prefix.push_str("$proc.StartInfo.Arguments = \"`\"$from`\" `\"$to`\" /e /move\"; ");
    prefix.push_str("$proc.StartInfo.RedirectStandardOutput = $true; ");
    prefix.push_str("$proc.StartInfo.RedirectStandardError = $true; ");
    prefix.push_str("$proc.StartInfo.UseShellExecute = $false; ");
    prefix.push_str(
        "$proc.StartInfo.WindowStyle = [System.Diagnostics.ProcessWindowStyle]::Hidden; ",
    );
    prefix.push_str("[void]$proc.Start(); $stdoutTask = $proc.StandardOutput.ReadToEndAsync(); $proc.WaitForExit(); ");
    prefix.push_str("if ($proc.ExitCode -ge 8) { throw \"Could not move '$from' into '$to' (robocopy exit $($proc.ExitCode)).\" }; ");
    prefix.push_str(
        "1..10 | ForEach-Object { if (Test-Path $from) { Start-Sleep -Milliseconds 100 } } }; ",
    );
    prefix.push_str("function Get-EnvVar { param([string]$Name,[switch]$Global) ");
    prefix.push_str("$registerKey = if ($Global) { Get-Item -Path 'HKLM:\\SYSTEM\\CurrentControlSet\\Control\\Session Manager' } else { Get-Item -Path 'HKCU:' }; ");
    prefix.push_str("$envRegisterKey = $registerKey.OpenSubKey('Environment'); ");
    prefix.push_str("$registryValueOption = [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames; ");
    prefix.push_str("$envRegisterKey.GetValue($Name, $null, $registryValueOption) }; ");
    prefix.push_str("function Set-EnvVar { param([string]$Name,[string]$Value,[switch]$Global) ");
    prefix.push_str("$registerKey = if ($Global) { Get-Item -Path 'HKLM:\\SYSTEM\\CurrentControlSet\\Control\\Session Manager' } else { Get-Item -Path 'HKCU:' }; ");
    prefix.push_str("$envRegisterKey = $registerKey.OpenSubKey('Environment', $true); ");
    prefix.push_str("if ($null -eq $Value -or $Value -eq '') { if ($envRegisterKey.GetValue($Name)) { $envRegisterKey.DeleteValue($Name) } } ");
    prefix.push_str("else { $registryValueKind = if ($Value.Contains('%')) { [Microsoft.Win32.RegistryValueKind]::ExpandString } elseif ($envRegisterKey.GetValue($Name)) { $envRegisterKey.GetValueKind($Name) } else { [Microsoft.Win32.RegistryValueKind]::String }; $envRegisterKey.SetValue($Name, $Value, $registryValueKind) } }; ");
    prefix.push_str("function Expand-MsiArchive { param([string]$Path,[string]$DestinationPath=(Split-Path $Path),[string]$ExtractDir,[string]$Switches,[switch]$Removal) ");
    prefix.push_str("$DestinationPath = $DestinationPath.TrimEnd('\\'); ");
    prefix.push_str("if ($ExtractDir) { $OriDestinationPath = $DestinationPath; $DestinationPath = \"$DestinationPath\\_tmp\" }; ");
    prefix
        .push_str("$ArgList = @('/a', $Path, '/qn', \"TARGETDIR=$DestinationPath\\SourceDir\"); ");
    prefix.push_str("if ($Switches) { $ArgList += (-split $Switches) }; ");
    prefix.push_str(&format!(
        "$status = Start-Process -FilePath '{}' -ArgumentList $ArgList -Wait -PassThru -WindowStyle Hidden; ",
        escape_powershell_literal(&msiexec_path().display().to_string())
    ));
    prefix.push_str("if ($status.ExitCode -ne 0) { abort \"Failed to extract files from $Path with msiexec exit code $($status.ExitCode).\" }; ");
    prefix.push_str("if ($ExtractDir -and (Test-Path \"$DestinationPath\\SourceDir\")) { movedir \"$DestinationPath\\SourceDir\\$ExtractDir\" $OriDestinationPath | Out-Null; Remove-Item $DestinationPath -Recurse -Force } ");
    prefix.push_str("elseif ($ExtractDir) { movedir \"$DestinationPath\\$ExtractDir\" $OriDestinationPath | Out-Null; Remove-Item $DestinationPath -Recurse -Force } ");
    prefix.push_str("elseif (Test-Path \"$DestinationPath\\SourceDir\") { movedir \"$DestinationPath\\SourceDir\" $DestinationPath | Out-Null }; ");
    prefix.push_str("if (($DestinationPath -ne (Split-Path $Path)) -and (Test-Path \"$DestinationPath\\$([System.IO.Path]::GetFileName($Path))\")) { Remove-Item \"$DestinationPath\\$([System.IO.Path]::GetFileName($Path))\" -Force }; ");
    prefix.push_str("if ($Removal) { Remove-Item $Path -Force } }; ");
    prefix.push_str("function Expand-DarkArchive { param([string]$Path,[string]$DestinationPath=(Split-Path $Path),[string]$Switches,[switch]$Removal) ");
    prefix.push_str("$dark = $env:SPOON_DARK_HELPER_PATH; if ([string]::IsNullOrWhiteSpace($dark)) { abort 'installer script requires installed helper ''dark''.' }; ");
    prefix.push_str("$ArgList = @('-nologo', '-x', $DestinationPath, $Path); if ($Switches) { $ArgList += (-split $Switches) }; ");
    prefix.push_str("$status = Start-Process -FilePath $dark -ArgumentList $ArgList -Wait -PassThru -WindowStyle Hidden; ");
    prefix.push_str("if ($status.ExitCode -ne 0) { abort \"Failed to extract files from $Path with dark exit code $($status.ExitCode).\" }; ");
    prefix.push_str("if (Test-Path \"$DestinationPath\\WixAttachedContainer\") { Rename-Item \"$DestinationPath\\WixAttachedContainer\" 'AttachedContainer' -ErrorAction Ignore } ");
    prefix.push_str("elseif (Test-Path \"$DestinationPath\\AttachedContainer\\a0\") { $Xml = [xml](Get-Content -Raw \"$DestinationPath\\UX\\manifest.xml\" -Encoding utf8); $Xml.BurnManifest.UX.Payload | ForEach-Object { Rename-Item \"$DestinationPath\\UX\\$($_.SourcePath)\" $_.FilePath -ErrorAction Ignore }; $Xml.BurnManifest.Payload | ForEach-Object { Rename-Item \"$DestinationPath\\AttachedContainer\\$($_.SourcePath)\" $_.FilePath -ErrorAction Ignore } }; ");
    prefix.push_str("if ($Removal) { Remove-Item $Path -Force } }; ");
    prefix.push_str("function Expand-InnoArchive { param([string]$Path,[string]$DestinationPath=(Split-Path $Path),[string]$ExtractDir,[string]$Switches,[switch]$Removal) ");
    prefix.push_str("$innounp = $env:SPOON_INNOUNP_HELPER_PATH; if ([string]::IsNullOrWhiteSpace($innounp)) { abort 'installer script requires installed helper ''innounp''.' }; ");
    prefix.push_str("$ArgList = @('-x', \"-d$DestinationPath\", $Path, '-y'); ");
    prefix.push_str("switch -Regex ($ExtractDir) { '^[^{].*' { $ArgList += \"-c{app}\\$ExtractDir\" } '^{.*' { $ArgList += \"-c$ExtractDir\" } Default { $ArgList += '-c{app}' } }; ");
    prefix.push_str("if ($Switches) { $ArgList += (-split $Switches) }; ");
    prefix.push_str("$status = Start-Process -FilePath $innounp -ArgumentList $ArgList -Wait -PassThru -WindowStyle Hidden; ");
    prefix.push_str("if ($status.ExitCode -ne 0) { abort \"Failed to extract files from $Path with innounp exit code $($status.ExitCode).\" }; ");
    prefix.push_str("if ($Removal) { Remove-Item $Path -Force } }; ");
    prefix.push_str(&format!(
        "$cmd='{}'; $dir='{}'; $persist_dir='{}'; $original_dir='{}'; $fname='{}'; $version='{}'; $architecture='{}'; $global=$false; ",
        escape_powershell_literal(command_name),
        escape_powershell_literal(&install_root.display().to_string()),
        escape_powershell_literal(&persist_root.display().to_string()),
        escape_powershell_literal(&install_root.display().to_string()),
        escape_powershell_literal(archive_name),
        escape_powershell_literal(version),
        selected_architecture_key(),
    ));
    if let Some(context) = context {
        prefix.push_str(&format!(
            "$app='{}'; ",
            escape_powershell_literal(&context.app)
        ));
        if let Some(bucket) = &context.bucket {
            prefix.push_str(&format!(
                "$bucket='{}'; ",
                escape_powershell_literal(bucket)
            ));
        }
        prefix.push_str(&format!(
            "$bucketsdir='{}'; ",
            escape_powershell_literal(&context.buckets_dir.display().to_string())
        ));
    }
    if let Some(path) = dark_helper_path {
        prefix.push_str(&format!(
            "$env:SPOON_DARK_HELPER_PATH='{}'; ",
            escape_powershell_literal(&path.display().to_string())
        ));
    }
    if let Some(path) = innounp_helper_path {
        prefix.push_str(&format!(
            "$env:SPOON_INNOUNP_HELPER_PATH='{}'; ",
            escape_powershell_literal(&path.display().to_string())
        ));
    }
    prefix
}

fn escape_powershell_literal(value: &str) -> String {
    value.replace('\'', "''")
}

fn build_hook_script(
    script: &str,
    install_root: &Path,
    persist_root: &Path,
    archive_path: Option<&Path>,
    context: Option<&HookContext>,
    command_name: &str,
    version: &str,
    dark_helper_path: Option<&Path>,
    innounp_helper_path: Option<&Path>,
) -> String {
    format!(
        "{}{}",
        render_hook_prelude(
            install_root,
            persist_root,
            archive_path,
            context,
            command_name,
            version,
            dark_helper_path,
            innounp_helper_path,
        ),
        script
    )
}

fn run_hook_script(
    script: &str,
    install_root: &Path,
    persist_root: &Path,
    archive_path: Option<&Path>,
    context: Option<&HookContext>,
    command_name: &str,
    version: &str,
    dark_helper_path: Option<&Path>,
    innounp_helper_path: Option<&Path>,
) -> Result<()> {
    let rendered = build_hook_script(
        script,
        install_root,
        persist_root,
        archive_path,
        context,
        command_name,
        version,
        dark_helper_path,
        innounp_helper_path,
    );
    let working_dir = if install_root.exists() {
        install_root
    } else if persist_root.exists() {
        persist_root
    } else {
        Path::new(".")
    };
    let mut command = Command::new("powershell");
    command
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(rendered)
        .current_dir(working_dir);
    let output = command.output().map_err(|err| {
        BackendError::external("failed to execute Scoop lifecycle hook script", err)
    })?;
    if output.status.success() {
        return Ok(());
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(BackendError::Other(format!(
        "Scoop lifecycle hook failed with status {:?}\nstdout: {}\nstderr: {}",
        output.status.code(),
        stdout,
        stderr
    )))
}

pub fn execute_hook_scripts(
    scripts: &[String],
    phase: &str,
    install_root: &Path,
    persist_root: &Path,
    archive_path: Option<&Path>,
    context: Option<&HookContext>,
    version: &str,
    dark_helper_path: Option<&Path>,
    innounp_helper_path: Option<&Path>,
    _emit: &mut dyn FnMut(BackendEvent),
) -> Result<()> {
    if scripts.is_empty() {
        return Ok(());
    }
    let script_block = scripts.join("\n");
    tracing::info!(
        "Running {} hook script block ({} line(s)).",
        phase,
        scripts.len()
    );
    let command_name = if phase.contains("uninstall") {
        "uninstall"
    } else {
        "install"
    };
    if let Err(error) = run_hook_script(
        &script_block,
        install_root,
        persist_root,
        archive_path,
        context,
        command_name,
        version,
        dark_helper_path,
        innounp_helper_path,
    ) {
        if phase.contains("uninstall") {
            tracing::warn!("Hook warning (ignored during {}): {}", phase, error);
        } else {
            return Err(error);
        }
    }
    Ok(())
}
