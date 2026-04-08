use super::{Modal, Screen};
use crate::packages;

pub(super) fn help_lines_for_screen(screen: &Screen) -> Vec<String> {
    match screen {
        Screen::Tools(_) => vec![
            "Tools".into(),
            "".into(),
            "Unified tools view with status overview and install/update/uninstall actions.".into(),
            "".into(),
            "Keys:".into(),
            "  1/2             Jump to Configure/Tools".into(),
            "  Left/Right      Switch pages directly".into(),
            "  j/k or Up/Down  Move selection".into(),
            "  Enter           Open selected item detail".into(),
            "  Space           Toggle current tool".into(),
            "  a / m / p / c   Select all / missing / installed / clear".into(),
            "  i / u / x       Install / update / uninstall selection".into(),
            "  r               Refresh status and latest versions".into(),
            "  D               Open debug log viewer".into(),
            "  Esc             Return to Configure".into(),
            "  ?               Show help".into(),
            "  q               Quit immediately".into(),
        ],
        Screen::ConfigMenu { .. } => vec![
            "Configure".into(),
            "".into(),
            "Configure editor setup plus Spoon-owned package and tool config surfaces.".into(),
            "".into(),
            "Keys:".into(),
            "  1/2             Jump to Configure/Tools".into(),
            "  Left/Right      Switch pages directly".into(),
            "  j/k or Up/Down  Move selection".into(),
            "  Enter           Open selected form".into(),
            "  r               Refresh configure state".into(),
            "  D               Open debug log viewer".into(),
            "  Esc             Show quit confirmation".into(),
            "  ?               Show help".into(),
            "  q               Quit immediately".into(),
        ],
    }
}

pub(super) fn help_lines_for_modal(modal: &Modal) -> Vec<String> {
    match modal {
        Modal::ToolDetail(_) => vec![
            "Tool Detail".into(),
            "".into(),
            "Shows summary, package info, config path, operation availability, and any side effects such as user PATH or subprocess environment changes for the selected item.".into(),
            "".into(),
            "Keys:".into(),
            "  Up/Down or j/k  Scroll detail".into(),
            "  c               Copy the full detail text".into(),
            "  i / u / x       Install / update / uninstall selected tool".into(),
            "  Enter or Esc    Close detail".into(),
            "  q               Quit immediately".into(),
        ],
        Modal::Form(form) => vec![
            format!("{} configuration", form.title),
            "".into(),
            "Read-only config view. spoon opens the backing config file in your editor.".into(),
            match form.kind {
                super::ConfigKind::Global => {
                    "Global opens the config file directly, including root, proxy, and editor settings.".into()
                }
                super::ConfigKind::Package(package_key)
                    if packages::config_target_descriptor(package_key)
                        .is_some_and(|descriptor| descriptor.editor_opens_parent_dir) =>
                {
                    "This target opens the config folder and the selected file together.".into()
                }
                super::ConfigKind::Package(package_key)
                    if packages::config_target_descriptor(package_key)
                        .is_some_and(|descriptor| descriptor.editable) =>
                {
                    "This target opens its native config file directly.".into()
                }
                super::ConfigKind::Package(_) => {
                    "Some package views are read-only summaries today and do not yet have a native config file to open directly.".into()
                }
            },
            "".into(),
            "Keys:".into(),
            "  Enter / e        Open config file in editor".into(),
            "  o                Reveal config file in Explorer".into(),
            "  Esc              Close configuration".into(),
            "  q                Quit immediately".into(),
        ],
        Modal::EditorSetup(_) => vec![
            "Editor Setup".into(),
            "".into(),
            "Install a free editor and manage the editor command spoon uses for config files."
                .into(),
            "".into(),
            "Keys:".into(),
            "  Up/Down          Move selection".into(),
            "  Enter / i        Install selected editor, or set it as default if ready".into(),
            "  u                Uninstall selected editor".into(),
            "  x                Clear default editor command".into(),
            "  Esc              Close setup".into(),
            "  q                Quit immediately".into(),
        ],
        Modal::Output(output) => vec![
            output.title.clone(),
            "".into(),
            if output.running {
                "Shows running command output until completion, and lets you copy the full log at any time.".into()
            } else {
                "Shows command output or confirmation details, and lets you copy the full log.".into()
            },
            "".into(),
            "Keys:".into(),
            "  Up/Down or j/k   Scroll output".into(),
            "  c                Copy the full output log".into(),
            if !output.running {
                "  Enter/Esc        Close output popup".into()
            } else {
                "  Enter/Esc        Wait for completion".into()
            },
            "  q                Quit immediately".into(),
        ],
        Modal::DebugLog(_) => vec![
            "Debug Log".into(),
            "".into(),
            "Interactive in-app log viewer for tracing background status checks, actions, and render timing.".into(),
            "".into(),
            "Keys:".into(),
            "  Up/Down          Move target selection".into(),
            "  Left/Right       Adjust shown log level for selected target".into(),
            "  - / +            Adjust captured log level for selected target".into(),
            "  h                Hide or show target selector".into(),
            "  f                Focus on selected target only".into(),
            "  Space            Hide targets with logging off".into(),
            "  PageUp/PageDown  Scroll log history".into(),
            "  Esc              Close debug log".into(),
            "  q                Quit immediately".into(),
        ],
        Modal::Help(help) => vec![
            help.title.clone(),
            "".into(),
            "Help popup for the current page or modal.".into(),
            "".into(),
            "Keys:".into(),
            "  Up/Down or j/k   Scroll help".into(),
            "  Enter/Esc/?      Close help".into(),
            "  q               Quit immediately".into(),
        ],
        Modal::QuitConfirm => vec![
            "Quit".into(),
            "".into(),
            "Confirm exiting spoon.".into(),
            "".into(),
            "Keys:".into(),
            "  Enter / y / q    Confirm exit".into(),
            "  Esc / n          Cancel".into(),
        ],
        Modal::CancelRunningConfirm(_) => vec![
            "Cancel Action".into(),
            "".into(),
            "Confirm cancelling the running action.".into(),
            "".into(),
            "Keys:".into(),
            "  Enter / y / q    Cancel the running action".into(),
            "  Esc / n          Keep it running".into(),
        ],
    }
}
