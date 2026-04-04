$ErrorActionPreference='Continue'
if (Get-Variable -Name PSNativeCommandUseErrorActionPreference -ErrorAction SilentlyContinue) { $PSNativeCommandUseErrorActionPreference = $false }

function ensure([string]$p) { if (-not (Test-Path -LiteralPath $p)) { New-Item -ItemType Directory -Path $p -Force | Out-Null }; Resolve-Path -LiteralPath $p }
function info([string]$m) { Write-Host "INFO  $m" -ForegroundColor DarkGray }
function warn([string]$m) { Write-Host "WARN  $m" -ForegroundColor DarkYellow }
function error([string]$m) { Write-Host "ERROR $m" -ForegroundColor DarkRed }
function abort([string]$m) { throw $m }
function movedir([string]$from,[string]$to) {
  $from = $from.TrimEnd('\\')
  $to = $to.TrimEnd('\\')
  $proc = New-Object System.Diagnostics.Process
  $proc.StartInfo.FileName = 'robocopy.exe'
  $proc.StartInfo.Arguments = "`"$from`" `"$to`" /e /move"
  $proc.StartInfo.RedirectStandardOutput = $true
  $proc.StartInfo.RedirectStandardError = $true
  $proc.StartInfo.UseShellExecute = $false
  $proc.StartInfo.WindowStyle = [System.Diagnostics.ProcessWindowStyle]::Hidden
  [void]$proc.Start()
  $stdoutTask = $proc.StandardOutput.ReadToEndAsync()
  $proc.WaitForExit()
  if ($proc.ExitCode -ge 8) { throw "Could not move '$from' into '$to' (robocopy exit $($proc.ExitCode))." }
  1..10 | ForEach-Object { if (Test-Path $from) { Start-Sleep -Milliseconds 100 } }
}
function Get-EnvVar { param([string]$Name,[switch]$Global)
  $registerKey = if ($Global) { Get-Item -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager' } else { Get-Item -Path 'HKCU:' }
  $envRegisterKey = $registerKey.OpenSubKey('Environment')
  $registryValueOption = [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames
  $envRegisterKey.GetValue($Name, $null, $registryValueOption)
}
function Set-EnvVar { param([string]$Name,[string]$Value,[switch]$Global)
  $registerKey = if ($Global) { Get-Item -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager' } else { Get-Item -Path 'HKCU:' }
  $envRegisterKey = $registerKey.OpenSubKey('Environment', $true)
  if ($null -eq $Value -or $Value -eq '') {
    if ($envRegisterKey.GetValue($Name)) { $envRegisterKey.DeleteValue($Name) }
  } else {
    $registryValueKind = if ($Value.Contains('%')) { [Microsoft.Win32.RegistryValueKind]::ExpandString } elseif ($envRegisterKey.GetValue($Name)) { $envRegisterKey.GetValueKind($Name) } else { [Microsoft.Win32.RegistryValueKind]::String }
    $envRegisterKey.SetValue($Name, $Value, $registryValueKind)
  }
}
function Expand-MsiArchive { param([string]$Path,[string]$DestinationPath=(Split-Path $Path),[string]$ExtractDir,[string]$Switches,[switch]$Removal)
  $DestinationPath = $DestinationPath.TrimEnd('\')
  if ($ExtractDir) { $OriDestinationPath = $DestinationPath; $DestinationPath = "$DestinationPath\_tmp" }
  $ArgList = @('/a', $Path, '/qn', "TARGETDIR=$DestinationPath\SourceDir")
  if ($Switches) { $ArgList += (-split $Switches) }
  $status = Start-Process -FilePath '{{ msiexec_path|ps }}' -ArgumentList $ArgList -Wait -PassThru -WindowStyle Hidden
  if ($status.ExitCode -ne 0) { abort "Failed to extract files from $Path with msiexec exit code $($status.ExitCode)." }
  if ($ExtractDir -and (Test-Path "$DestinationPath\SourceDir")) { movedir "$DestinationPath\SourceDir\$ExtractDir" $OriDestinationPath | Out-Null; Remove-Item $DestinationPath -Recurse -Force }
  elseif ($ExtractDir) { movedir "$DestinationPath\$ExtractDir" $OriDestinationPath | Out-Null; Remove-Item $DestinationPath -Recurse -Force }
  elseif (Test-Path "$DestinationPath\SourceDir") { movedir "$DestinationPath\SourceDir" $DestinationPath | Out-Null }
  if (($DestinationPath -ne (Split-Path $Path)) -and (Test-Path "$DestinationPath\$([System.IO.Path]::GetFileName($Path))")) { Remove-Item "$DestinationPath\$([System.IO.Path]::GetFileName($Path))" -Force }
  if ($Removal) { Remove-Item $Path -Force }
}
function Expand-DarkArchive { param([string]$Path,[string]$DestinationPath=(Split-Path $Path),[string]$Switches,[switch]$Removal)
  $dark = $env:SPOON_DARK_HELPER_PATH
  if ([string]::IsNullOrWhiteSpace($dark)) { abort "installer script requires installed helper 'dark'." }
  $ArgList = @('-nologo', '-x', $DestinationPath, $Path)
  if ($Switches) { $ArgList += (-split $Switches) }
  $status = Start-Process -FilePath $dark -ArgumentList $ArgList -Wait -PassThru -WindowStyle Hidden
  if ($status.ExitCode -ne 0) { abort "Failed to extract files from $Path with dark exit code $($status.ExitCode)." }
  if (Test-Path "$DestinationPath\WixAttachedContainer") { Rename-Item "$DestinationPath\WixAttachedContainer" 'AttachedContainer' -ErrorAction Ignore }
  elseif (Test-Path "$DestinationPath\AttachedContainer\a0") { $Xml = [xml](Get-Content -Raw "$DestinationPath\UX\manifest.xml" -Encoding utf8); $Xml.BurnManifest.UX.Payload | ForEach-Object { Rename-Item "$DestinationPath\UX\$($_.SourcePath)" $_.FilePath -ErrorAction Ignore }; $Xml.BurnManifest.Payload | ForEach-Object { Rename-Item "$DestinationPath\AttachedContainer\$($_.SourcePath)" $_.FilePath -ErrorAction Ignore } }
  if ($Removal) { Remove-Item $Path -Force }
}
function Expand-InnoArchive { param([string]$Path,[string]$DestinationPath=(Split-Path $Path),[string]$ExtractDir,[string]$Switches,[switch]$Removal)
  $innounp = $env:SPOON_INNOUNP_HELPER_PATH
  if ([string]::IsNullOrWhiteSpace($innounp)) { abort "installer script requires installed helper 'innounp'." }
  $ArgList = @('-x', "-d$DestinationPath", $Path, '-y')
  switch -Regex ($ExtractDir) {
    '^[^{].*' { $ArgList += "-c{app}\$ExtractDir" }
    '^{.*' { $ArgList += "-c$ExtractDir" }
    Default { $ArgList += '-c{app}' }
  }
  if ($Switches) { $ArgList += (-split $Switches) }
  $status = Start-Process -FilePath $innounp -ArgumentList $ArgList -Wait -PassThru -WindowStyle Hidden
  if ($status.ExitCode -ne 0) { abort "Failed to extract files from $Path with innounp exit code $($status.ExitCode)." }
  if ($Removal) { Remove-Item $Path -Force }
}

$cmd='{{ command_name|ps }}'
$dir='{{ install_root|ps }}'
$persist_dir='{{ persist_root|ps }}'
$original_dir='{{ install_root|ps }}'
$fname='{{ archive_name|ps }}'
$version='{{ version|ps }}'
$architecture='{{ architecture|ps }}'
$global=$false
{% if context_app %}
$app='{{ context_app|ps }}'
{% endif %}
{% if context_bucket %}
$bucket='{{ context_bucket|ps }}'
{% endif %}
{% if context_buckets_dir %}
$bucketsdir='{{ context_buckets_dir|ps }}'
{% endif %}
{% if dark_helper_path %}
$env:SPOON_DARK_HELPER_PATH='{{ dark_helper_path|ps }}'
{% endif %}
{% if innounp_helper_path %}
$env:SPOON_INNOUNP_HELPER_PATH='{{ innounp_helper_path|ps }}'
{% endif %}
