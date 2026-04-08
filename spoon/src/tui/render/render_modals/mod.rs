mod confirm;
mod debug;
mod detail;
mod forms;
mod utility;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::tui::{Modal, Screen};

pub(super) fn render_modal(
    buf: &mut Buffer,
    area: Rect,
    screen: &Screen,
    modal: &mut Modal,
    transient_hint: Option<&str>,
) {
    match modal {
        Modal::ToolDetail(detail) => detail::render_tool_detail_modal(buf, area, screen, detail),
        Modal::Form(form) => forms::render_form_modal(buf, area, form, transient_hint),
        Modal::EditorSetup(setup) => {
            forms::render_editor_setup_modal(buf, area, setup, transient_hint);
        }
        Modal::Output(output) => {
            utility::render_output_modal(buf, area, output, transient_hint);
        }
        Modal::DebugLog(debug_log) => {
            debug::render_debug_log_modal(buf, area, debug_log, transient_hint);
        }
        Modal::Help(help) => {
            utility::render_help_modal(buf, area, help);
        }
        Modal::CancelRunningConfirm(_) => confirm::render_cancel_running_modal(buf, area),
        Modal::QuitConfirm => confirm::render_quit_modal(buf, area),
    }
}
