# Refresh PATH from the Windows registry and run any command.
# Usage: powershell -File <this_script> <command> <args...>
#
# Why: In Claude Code, bash inherits PATH from VSCode (the parent process),
# not from the registry. After installing software, the new PATH entries are
# only in the registry. This script reads the fresh PATH before running the command.

$env:Path = [Environment]::GetEnvironmentVariable("PATH", "User") + ";" + [Environment]::GetEnvironmentVariable("PATH", "Machine")
$cmd = $args[0]
[array]$cmdArgs = $args[1..($args.Length - 1)]
& $cmd @cmdArgs
