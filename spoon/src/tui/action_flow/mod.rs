mod config;
mod editor_setup;
mod tools;

use tokio::sync::mpsc::UnboundedSender;

use crate::status;

use super::{ActionOutcome, BackgroundEvent};

pub(crate) use config::{FormOutcome, handle_form_key, open_config_target_modal};
pub(crate) use editor_setup::{EditorSetupOutcome, handle_editor_setup_key};
pub(crate) use tools::{ToolsActionStart, start_tools_action};

pub(crate) fn complete_background_action(
    tx: &UnboundedSender<BackgroundEvent>,
    mut outcome: ActionOutcome,
    refresh_env: bool,
) {
    if refresh_env && let Err(err) = status::refresh_process_env_from_registry() {
        if outcome.append_lines {
            outcome.lines.push(format!(
                "Warning: failed to refresh PATH from registry: {err}"
            ));
        } else {
            outcome.lines = vec![format!(
                "Warning: failed to refresh PATH from registry: {err}"
            )];
        }
    }
    let _ = tx.send(BackgroundEvent::Complete(outcome));
}
