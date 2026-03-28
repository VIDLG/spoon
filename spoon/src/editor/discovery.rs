use std::path::{Path, PathBuf};

use crate::config;
use crate::editor::model::{EDITOR_CANDIDATES, EditorCandidate, EditorStatus};
use crate::editor::state;
use spoon_backend::layout::RuntimeLayout;

pub fn candidates() -> &'static [EditorCandidate] {
    &EDITOR_CANDIDATES
}

pub fn recommended_candidate_index() -> usize {
    0
}

pub fn default_editor_status() -> EditorStatus {
    let command = resolve_default_editor_command();
    let executable = extract_executable(&command);
    let available = command_available(&executable);
    EditorStatus {
        command,
        executable,
        available,
    }
}

pub fn is_candidate_available(candidate: EditorCandidate) -> bool {
    if let Some(available) = state::availability_override(candidate.command) {
        return available;
    }
    if state::test_mode_enabled()
        && let Some(available) = state::test_candidate_availability()
    {
        return available;
    }
    command_available(candidate.command)
}

pub fn is_default_candidate(candidate: EditorCandidate) -> bool {
    let global = config::load_global_config();
    global.editor.trim().eq_ignore_ascii_case(candidate.command)
}

pub fn is_candidate_managed(candidate: EditorCandidate) -> bool {
    managed_scoop_editor_path(candidate).is_some()
}

pub fn is_candidate_external(candidate: EditorCandidate) -> bool {
    is_candidate_available(candidate) && !is_candidate_managed(candidate)
}

pub fn resolve_default_editor_command() -> String {
    let global = config::load_global_config();
    if !global.editor.trim().is_empty() {
        return global.editor.trim().to_string();
    }
    if let Ok(env_editor) = std::env::var("EDITOR")
        && !env_editor.trim().is_empty()
    {
        return env_editor.trim().to_string();
    }
    EDITOR_CANDIDATES[0].command.to_string()
}

pub fn command_available(command: &str) -> bool {
    if command.is_empty() {
        return false;
    }
    if let Some(available) = state::availability_override(command) {
        return available;
    }
    let path = Path::new(command);
    if path.is_absolute() || command.contains('\\') || command.contains('/') {
        return path.exists();
    }
    command_path(command).is_some()
}

pub fn command_path(command: &str) -> Option<PathBuf> {
    which::which(command).ok()
}

pub fn extract_executable(command: &str) -> String {
    let trimmed = command.trim();
    if let Some(rest) = trimmed.strip_prefix('"')
        && let Some((exe, _)) = rest.split_once('"')
    {
        return exe.to_string();
    }
    trimmed.split_whitespace().next().unwrap_or("").to_string()
}

pub fn split_command(command: &str) -> (String, String) {
    let trimmed = command.trim();
    if let Some(rest) = trimmed.strip_prefix('"')
        && let Some((exe, tail)) = rest.split_once('"')
    {
        return (exe.to_string(), tail.trim().to_string());
    }
    if let Some((exe, tail)) = trimmed.split_once(char::is_whitespace) {
        (exe.to_string(), tail.trim().to_string())
    } else {
        (trimmed.to_string(), String::new())
    }
}

fn managed_scoop_editor_path(candidate: EditorCandidate) -> Option<PathBuf> {
    let tool_root = config::configured_tool_root()?;
    let layout = RuntimeLayout::from_root(&tool_root);

    let mut candidates = Vec::new();
    for ext in ["", ".cmd", ".exe", ".bat"] {
        let name = format!("{}{}", candidate.command, ext);
        candidates.push(
            layout
                .scoop
                .apps_root
                .join(candidate.package_name)
                .join("current")
                .join("bin")
                .join(&name),
        );
        candidates.push(
            layout
                .scoop
                .apps_root
                .join(candidate.package_name)
                .join("current")
                .join(&name),
        );
        candidates.push(layout.shims.join(&name));
    }

    candidates.into_iter().find(|path| path.exists())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use crate::config::{self, save_global_config};

    use super::{EditorCandidate, is_candidate_managed, managed_scoop_editor_path};

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "spoon-{prefix}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn managed_editor_detection_accepts_scoop_managed_editor_path() {
        crate::config::enable_test_mode();
        let tool_root = unique_temp_dir("editor-managed-scoop");
        let code_dir = tool_root
            .join("scoop")
            .join("apps")
            .join("vscode")
            .join("current")
            .join("bin");
        fs::create_dir_all(&code_dir).unwrap();
        let code_cmd = code_dir.join("code.cmd");
        fs::write(&code_cmd, "@echo off\r\n").unwrap();

        let original = config::load_global_config();
        let mut updated = original.clone();
        updated.root = tool_root.display().to_string();
        save_global_config(&updated).unwrap();

        let candidate = EditorCandidate {
            label: "VS Code",
            command: "code",
            package_name: "vscode",
        };
        assert_eq!(
            managed_scoop_editor_path(candidate).as_deref(),
            Some(code_cmd.as_path())
        );
        assert!(is_candidate_managed(candidate));

        save_global_config(&original).unwrap();
    }
}
