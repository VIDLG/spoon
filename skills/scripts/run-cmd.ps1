# Refresh all environment variables from the Windows registry and run a command.
# Usage: powershell -File <this_script> <command> <args...>
#
# Why: In Claude Code, bash inherits env vars from VSCode (the parent process),
# not from the registry. After installing software, new variables (PATH, JAVA_HOME,
# GOPATH, etc.) are only in the registry. This script reads them all before running
# the command.

# Apply Machine-level variables first
foreach ($entry in [Environment]::GetEnvironmentVariables("Machine").GetEnumerator()) {
    if ($entry.Key -ne "Path") {
        [Environment]::SetEnvironmentVariable($entry.Key, $entry.Value, "Process")
    }
}

# Apply User-level variables (override Machine for non-PATH)
foreach ($entry in [Environment]::GetEnvironmentVariables("User").GetEnumerator()) {
    if ($entry.Key -ne "Path") {
        [Environment]::SetEnvironmentVariable($entry.Key, $entry.Value, "Process")
    }
}

# PATH is special: concatenate User + Machine
$env:Path = [Environment]::GetEnvironmentVariable("Path", "User") + ";" + [Environment]::GetEnvironmentVariable("Path", "Machine")

$cmd = $args[0]
[array]$cmdArgs = $args[1..($args.Length - 1)]
& $cmd @cmdArgs
