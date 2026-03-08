# Add or remove subdirectories of a scoop app from user PATH.
# Usage:
#   powershell -File <this_script> <app> <subdir1> [subdir2] ...
#   powershell -File <this_script> -Remove <app> <subdir1> [subdir2] ...
#
# Examples:
#   powershell -File add-path.ps1 git bin usr/bin
#   powershell -File add-path.ps1 python Scripts
#   powershell -File add-path.ps1 -Remove git bin usr/bin
#
# Each <subdir> is relative to <SCOOP>/apps/<app>/current/.

param(
    [switch]$Remove
)

$scoop = [Environment]::GetEnvironmentVariable("SCOOP", "User")
if (-not $scoop) {
    Write-Host "SCOOP environment variable not set"
    exit 1
}

# First positional arg is the app name, rest are subdirectories
$app = $args[0]
$subdirs = $args[1..($args.Length - 1)]

if (-not $app -or $subdirs.Count -eq 0) {
    Write-Host "Usage: add-path.ps1 [-Remove] <app> <subdir1> [subdir2] ..."
    exit 1
}

$appBase = "$scoop\apps\$app\current"
$targetPaths = $subdirs | ForEach-Object { Join-Path $appBase $_ }
$path = [Environment]::GetEnvironmentVariable("PATH", "User")

if ($Remove) {
    $entries = $path -split ";"
    $cleaned = ($entries | Where-Object {
        $entry = $_
        -not ($targetPaths | Where-Object { $entry -eq $_ })
    }) -join ";"
    [Environment]::SetEnvironmentVariable("PATH", $cleaned, "User")
    Write-Host "Removed from PATH: $($targetPaths -join ', ')"
} else {
    $newEntries = @()
    foreach ($tp in $targetPaths) {
        if ($path -notmatch [regex]::Escape($tp)) {
            $newEntries += $tp
        }
    }
    if ($newEntries.Count -gt 0) {
        [Environment]::SetEnvironmentVariable("PATH", ($path + ";" + ($newEntries -join ";")), "User")
        Write-Host "Added to PATH: $($newEntries -join ', ')"
    } else {
        Write-Host "Already in PATH"
    }
}
