use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::thread;
use std::time::{Duration, SystemTime};

use bytesize::ByteSize;

const DEPLOY_WAIT_ATTEMPTS: u64 = 5;
const DEPLOY_WAIT_STEP: Duration = Duration::from_secs(1);

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("xtask error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "help".to_string());

    match command.as_str() {
        "deploy" => deploy(),
        "help" | "-h" | "--help" => {
            print_help();
            Ok(())
        }
        other => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unknown xtask command: {other}"),
        )),
    }
}

fn deploy() -> io::Result<()> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "failed to locate workspace root")
        })?;
    let spoon_crate = workspace_root.join("spoon");
    let build_target = detect_current_build_target(&workspace_root)?;

    let mut build_args = vec!["build", "--release", "--bin", "spoon", "-p", "spoon"];
    if let Some(target) = build_target.as_deref() {
        build_args.push("--target");
        build_args.push(target);
    }

    run_checked("cargo", &build_args, &workspace_root)?;

    let source = release_binary_path(&workspace_root, build_target.as_deref());
    let repo_dest = workspace_root.join("spoon.exe");
    let mut deploy_targets = vec![repo_dest];
    let path_target = user_local_bin_deploy_target(&workspace_root);
    if !deploy_targets.contains(&path_target) {
        deploy_targets.push(path_target);
    }

    if !spoon_crate.join("Cargo.toml").exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "failed to locate spoon crate under {}",
                spoon_crate.display()
            ),
        ));
    }

    for dest in &deploy_targets {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        replace_in_place(&source, dest)?;
        cleanup_deploy_artifacts(dest);
        print_metadata(dest)?;
    }
    Ok(())
}

fn user_home_dir(workspace_root: &Path) -> PathBuf {
    resolve_user_home(
        env::var_os("SPOON_TEST_HOME").map(PathBuf::from),
        env::var_os("USERPROFILE").map(PathBuf::from),
        env::var_os("HOME").map(PathBuf::from),
        workspace_root,
    )
}

fn user_local_bin_deploy_target(workspace_root: &Path) -> PathBuf {
    user_home_dir(workspace_root)
        .join(".local")
        .join("bin")
        .join("spoon.exe")
}

fn resolve_user_home(
    spoon_test_home: Option<PathBuf>,
    user_profile: Option<PathBuf>,
    home: Option<PathBuf>,
    workspace_root: &Path,
) -> PathBuf {
    spoon_test_home
        .or(user_profile)
        .or(home)
        .unwrap_or_else(|| workspace_root.to_path_buf())
}

fn detect_current_build_target(workspace_root: &Path) -> io::Result<Option<String>> {
    let current_exe = env::current_exe()?;
    let target_root = workspace_root.join("target");
    let Ok(relative) = current_exe.strip_prefix(&target_root) else {
        return Ok(None);
    };
    let mut components = relative.components();
    let Some(first) = components.next() else {
        return Ok(None);
    };
    let first = first.as_os_str().to_string_lossy().to_string();
    if matches!(first.as_str(), "debug" | "release") {
        return Ok(None);
    }
    Ok(Some(first))
}

fn release_binary_path(workspace_root: &Path, target: Option<&str>) -> PathBuf {
    let mut path = workspace_root.join("target");
    if let Some(target) = target {
        path = path.join(target);
    }
    path.join("release").join("spoon.exe")
}

fn replace_in_place(source: &Path, dest: &Path) -> io::Result<()> {
    match fs::copy(source, dest) {
        Ok(_) => {
            cleanup_deploy_artifacts(dest);
            Ok(())
        }
        Err(err) if is_lock_error(&err) => {
            wait_for_manual_close(source, dest)?;
            if fs::copy(source, dest).is_ok() {
                cleanup_deploy_artifacts(dest);
                return Ok(());
            }

            stop_spoon_processes()?;
            match fs::copy(source, dest) {
                Ok(_) => {
                    cleanup_deploy_artifacts(dest);
                    Ok(())
                }
                Err(err) if is_lock_error(&err) => Err(io::Error::new(
                    err.kind(),
                    format!(
                        "failed to replace {} because it is still running; stop spoon.exe and retry",
                        dest.display()
                    ),
                )),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

fn wait_for_manual_close(source: &Path, dest: &Path) -> io::Result<()> {
    println!(
        "spoon.exe is in use. Waiting up to {}s for it to close...",
        DEPLOY_WAIT_ATTEMPTS
    );
    for second in 1..=DEPLOY_WAIT_ATTEMPTS {
        thread::sleep(DEPLOY_WAIT_STEP);
        match fs::copy(source, dest) {
            Ok(_) => {
                cleanup_deploy_artifacts(dest);
                println!("spoon.exe was released after {second}s.");
                return Ok(());
            }
            Err(err) if is_lock_error(&err) => {
                println!("  still locked after {second}s...");
            }
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

fn stop_spoon_processes() -> io::Result<()> {
    let status = Command::new("taskkill")
        .args(["/IM", "spoon.exe", "/F"])
        .status()?;
    if status.success() {
        return Ok(());
    }
    Ok(())
}

fn is_lock_error(err: &io::Error) -> bool {
    matches!(err.raw_os_error(), Some(5) | Some(32))
}

fn run_checked(program: &str, args: &[&str], cwd: &Path) -> io::Result<()> {
    let status = Command::new(program).args(args).current_dir(cwd).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "{program} {:?} failed with status {status}",
            args
        )))
    }
}

fn print_metadata(path: &Path) -> io::Result<()> {
    let meta = fs::metadata(path)?;
    let modified = meta.modified().ok();
    println!("deployed: {}", path.display());
    println!("size: {}", format_bytes(meta.len()));
    if let Some(modified) = modified {
        println!("modified: {}", format_system_time(modified));
    }
    Ok(())
}

fn format_bytes(size: u64) -> String {
    ByteSize(size).display().iec().to_string()
}

fn format_system_time(time: SystemTime) -> String {
    humantime::format_rfc3339_seconds(time).to_string()
}

fn cleanup_deploy_artifacts(dest: &Path) {
    let _ = fs::remove_file(dest.with_extension("next.exe"));
    let _ = fs::remove_file(dest.with_extension("deploy-status.txt"));
}

fn print_help() {
    println!("xtask commands:");
    println!(
        "  deploy    Build release spoon and replace repository-root spoon.exe plus ~/.local/bin/spoon.exe"
    );
}

#[cfg(test)]
mod tests {
    use super::{release_binary_path, resolve_user_home};
    use std::path::{Path, PathBuf};

    #[test]
    fn release_binary_path_supports_targeted_builds() {
        let workspace = Path::new(r"D:\projects\spoon");
        assert_eq!(
            release_binary_path(workspace, Some("x86_64-pc-windows-gnu")),
            Path::new(r"D:\projects\spoon\target\x86_64-pc-windows-gnu\release\spoon.exe")
        );
        assert_eq!(
            release_binary_path(workspace, None),
            Path::new(r"D:\projects\spoon\target\release\spoon.exe")
        );
    }

    #[test]
    fn resolve_user_home_prefers_spoon_test_home_then_profile_then_home() {
        let workspace = Path::new(r"D:\projects\spoon");
        let temp_home = PathBuf::from(r"C:\temp\spoon-home");
        let user_profile = PathBuf::from(r"C:\Users\vision");
        let home = PathBuf::from(r"C:\Users\vision-home");

        assert_eq!(
            resolve_user_home(
                Some(temp_home.clone()),
                Some(user_profile.clone()),
                Some(home.clone()),
                workspace,
            ),
            temp_home
        );
        assert_eq!(
            resolve_user_home(
                None,
                Some(user_profile.clone()),
                Some(home.clone()),
                workspace
            ),
            user_profile
        );
        assert_eq!(
            resolve_user_home(None, None, Some(home.clone()), workspace),
            home
        );
        assert_eq!(
            resolve_user_home(None, None, None, workspace),
            workspace.to_path_buf()
        );
    }

    #[test]
    fn user_local_bin_target_comes_from_resolved_home() {
        let resolved = resolve_user_home(
            Some(PathBuf::from(r"C:\temp\spoon-home")),
            None,
            None,
            Path::new(r"D:\projects\spoon"),
        );
        assert_eq!(
            resolved.join(".local").join("bin").join("spoon.exe"),
            Path::new(r"C:\temp\spoon-home\.local\bin\spoon.exe")
        );
    }
}
