use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;

use crate::platform::shell;

static TEST_MODE: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct LaunchResult {
    pub pid: Option<u32>,
}

pub fn enable_test_mode() {
    TEST_MODE.store(true, Ordering::Relaxed);
}

pub fn open_in_editor(command_line: &str) -> Result<LaunchResult> {
    if TEST_MODE.load(Ordering::Relaxed) {
        return Ok(LaunchResult { pid: Some(0) });
    }

    let child = shell::cmd().args(["/C", command_line]).spawn()?;
    Ok(LaunchResult {
        pid: Some(child.id()),
    })
}

pub fn open_program(program: &str, args: &[String], cwd: Option<&Path>) -> Result<LaunchResult> {
    if TEST_MODE.load(Ordering::Relaxed) {
        return Ok(LaunchResult { pid: Some(0) });
    }

    let mut command = shell::direct(program);
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let child = command.spawn()?;
    Ok(LaunchResult {
        pid: Some(child.id()),
    })
}

pub fn open_program_in_new_terminal(program: &str, args: &[String]) -> Result<LaunchResult> {
    if TEST_MODE.load(Ordering::Relaxed) {
        return Ok(LaunchResult { pid: Some(0) });
    }

    let mut command = shell::cmd_start(program, args);
    let child = command.spawn()?;
    Ok(LaunchResult {
        pid: Some(child.id()),
    })
}

pub fn reveal_in_explorer(path: &Path) -> Result<LaunchResult> {
    if TEST_MODE.load(Ordering::Relaxed) {
        return Ok(LaunchResult { pid: Some(0) });
    }

    let explorer_arg = format!("/select,{}", path.display());
    let child = shell::direct("explorer.exe").arg(&explorer_arg).spawn()?;
    Ok(LaunchResult {
        pid: Some(child.id()),
    })
}
