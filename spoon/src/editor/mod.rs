mod discovery;
mod launch;
mod manage;
mod model;
mod state;

pub use discovery::{
    candidates, command_available, command_path, default_editor_status, extract_executable,
    is_candidate_available, is_candidate_external, is_candidate_managed, is_default_candidate,
    recommended_candidate_index, resolve_default_editor_command, split_command,
};
pub(crate) use launch::open_file_in_default_editor;
pub use launch::{
    EditorLaunchResult, build_editor_command, format_command_for_display, is_terminal_editor,
};
pub use manage::{apply_candidate, clear_default_editor, install_candidate, uninstall_candidate};
pub(crate) use manage::{install_candidate_streaming, uninstall_candidate_streaming};
pub use model::{EditorCandidate, EditorStatus};
pub use state::{
    enable_test_mode, reset_availability_overrides, set_test_candidate_availability,
    test_mode_enabled,
};

#[cfg(test)]
mod tests {
    use super::launch::launch_args_for_simple_editor;
    use super::{is_terminal_editor, split_command};
    use crate::tui::ConfigKind;
    use std::path::Path;

    #[test]
    fn split_command_handles_simple_executable() {
        let (program, rest) = split_command("zed");
        assert_eq!(program, "zed");
        assert!(rest.is_empty());
    }

    #[test]
    fn split_command_handles_quoted_path_and_args() {
        let (program, rest) = split_command(r#""X:\Program Files\Editor\editor.exe" -w"#);
        assert_eq!(program, r#"X:\Program Files\Editor\editor.exe"#);
        assert_eq!(rest, "-w");
    }

    #[test]
    fn zed_opens_directory_and_file() {
        let args = launch_args_for_simple_editor(
            ConfigKind::Package("claude"),
            "zed",
            Path::new(r"X:\test-home\.claude\settings.json"),
        );
        assert_eq!(args[0], r"X:\test-home\.claude");
        assert_eq!(args[1], r"X:\test-home\.claude\settings.json");
    }

    #[test]
    fn nano_opens_file_only() {
        let args = launch_args_for_simple_editor(
            ConfigKind::Global,
            "nano",
            Path::new(r"X:\test-home\.spoon\config.toml"),
        );
        assert_eq!(args, vec![r"X:\test-home\.spoon\config.toml".to_string()]);
    }

    #[test]
    fn zed_opens_global_as_single_file() {
        let args = launch_args_for_simple_editor(
            ConfigKind::Global,
            "zed",
            Path::new(r"X:\test-home\.spoon\config.toml"),
        );
        assert_eq!(args, vec![r"X:\test-home\.spoon\config.toml".to_string()]);
    }

    #[test]
    fn nano_is_treated_as_terminal_editor() {
        assert!(is_terminal_editor("nano"));
        assert!(is_terminal_editor(r"X:\bin\nano.exe"));
        assert!(!is_terminal_editor("zed"));
    }
}
