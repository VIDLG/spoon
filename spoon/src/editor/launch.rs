use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::editor::discovery::{command_path, resolve_default_editor_command, split_command};
use crate::launcher;
use crate::packages;
use crate::tui::ConfigKind;

#[derive(Debug, Clone)]
pub struct EditorLaunchResult {
    pub command_line: String,
    pub pid: Option<u32>,
}

pub fn build_editor_command(file: &Path) -> String {
    let template = resolve_default_editor_command();
    let file_arg = format!("\"{}\"", file.display());
    if template.contains("{file}") {
        template.replace("{file}", &file_arg)
    } else {
        format!("{template} {file_arg}")
    }
}

pub(crate) fn open_file_in_default_editor(
    kind: ConfigKind,
    file: &Path,
) -> Result<EditorLaunchResult> {
    let template = resolve_default_editor_command();
    let (program, rest) = split_command(&template);
    let cwd = file.parent();
    let launch_program = command_path(&program)
        .unwrap_or_else(|| PathBuf::from(&program))
        .display()
        .to_string();

    if !template.contains("{file}") && !program.is_empty() && rest.is_empty() {
        let args = launch_args_for_simple_editor(kind, &program, file);
        let launch = if is_terminal_editor(&program) {
            launcher::open_program_in_new_terminal(&launch_program, &args)?
        } else {
            launcher::open_program(&launch_program, &args, cwd)?
        };
        return Ok(EditorLaunchResult {
            command_line: format_command_for_display(&launch_program, &args),
            pid: launch.pid,
        });
    }

    let command_line = build_editor_command(file);
    let launch = launcher::open_in_editor(&command_line)?;
    Ok(EditorLaunchResult {
        command_line,
        pid: launch.pid,
    })
}

pub(crate) fn launch_args_for_simple_editor(
    kind: ConfigKind,
    program: &str,
    file: &Path,
) -> Vec<String> {
    let name = Path::new(program)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(program)
        .to_ascii_lowercase();

    let open_dir_with_file = matches!(kind, ConfigKind::Package(package_key)
        if packages::config_target_descriptor(package_key)
            .is_some_and(|descriptor| descriptor.editor_opens_parent_dir))
        && matches!(name.as_str(), "zed" | "code" | "code-insiders" | "cursor");

    if open_dir_with_file {
        let mut args = Vec::new();
        if let Some(parent) = file.parent() {
            args.push(parent.display().to_string());
        }
        args.push(file.display().to_string());
        return args;
    }

    vec![file.display().to_string()]
}

pub fn is_terminal_editor(program: &str) -> bool {
    let name = Path::new(program)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(program)
        .to_ascii_lowercase();
    matches!(name.as_str(), "nano")
}

pub fn format_command_for_display(program: &str, args: &[String]) -> String {
    if args.is_empty() {
        return program.to_string();
    }
    let joined = args
        .iter()
        .map(|arg| format!(r#""{arg}""#))
        .collect::<Vec<_>>()
        .join(" ");
    format!("{program} {joined}")
}
