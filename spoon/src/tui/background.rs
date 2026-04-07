use crate::config;
use crate::logger;
use crate::runtime;
use crate::status;
use crate::status::ToolStatus;

use super::{ActionOutcome, App, BackgroundEvent, BgStatusUpdate, Modal, OutputState, Screen};

pub(crate) fn start_bg_status_check(app: &mut App) {
    logger::tui_background_status_check_start();
    let root = app.install_root.clone();
    let rx = runtime::spawn_with_sender(move |tx| {
        let _ = tx.send(BgStatusUpdate::Config(crate::tui::AppConfigSnapshot::load()));
        let resolved_root = root
            .as_deref()
            .map(|r| std::path::PathBuf::from(r))
            .or_else(config::configured_tool_root);
        let snapshot = resolved_root.as_deref().map(status::snapshot);
        let mut statuses =
            status::collect_statuses_with_snapshot(root.as_deref(), snapshot.as_ref());
        status::populate_installed_size_info(&mut statuses);
        let _ = tx.send(BgStatusUpdate::Statuses(statuses.clone()));
        status::populate_update_info(&mut statuses, root.as_deref());
        let _ = tx.send(BgStatusUpdate::Statuses(statuses));
    });
    app.bg_status_rx = Some(rx);
    app.status_hint = Some("Checking tool versions...".to_string());
}

pub(crate) fn poll_bg_status(app: &mut App) {
    let Some(rx) = app.bg_status_rx.as_mut() else {
        return;
    };

    match rx.try_recv() {
        Ok(BgStatusUpdate::Config(config_snapshot)) => {
            app.config_snapshot = config_snapshot;
        }
        Ok(BgStatusUpdate::Statuses(statuses)) => {
            let has_update_info = statuses
                .iter()
                .any(|s| s.update_available || s.latest_version.is_some());
            logger::tui_background_status_check_update(statuses.len(), has_update_info);
            app.statuses_snapshot = statuses.clone();
            apply_statuses_to_screen(&mut app.screen, statuses);
            if has_update_info {
                app.status_hint = None;
                app.bg_status_rx = None;
                logger::tui_background_status_check_complete();
            } else {
                app.status_hint = Some("Checking latest versions...".to_string());
            }
        }
        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {}
        Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
            app.status_hint = None;
            app.bg_status_rx = None;
            logger::tui_background_status_check_disconnected();
        }
    }
}

fn apply_statuses_to_screen(screen: &mut Screen, statuses: Vec<ToolStatus>) {
    if let Screen::Tools(state) = screen {
        state.apply_statuses(statuses);
    }
}

pub(crate) fn poll_background_action(app: &mut App) {
    let Some(background) = app.background_action.as_mut() else {
        return;
    };

    let mut outcome: Option<ActionOutcome> = None;
    loop {
        match background.rx.try_recv() {
            Ok(BackgroundEvent::AppendLine(line)) => {
                if let Some(Modal::Output(output)) = app.modal.as_mut() {
                    output.lines.push(line);
                    if output.auto_scroll {
                        output.snap_to_bottom_on_render = true;
                    }
                }
            }
            Ok(BackgroundEvent::ReplaceLastLine(line)) => {
                if let Some(Modal::Output(output)) = app.modal.as_mut() {
                    if let Some(last) = output.lines.last_mut() {
                        *last = line;
                    } else {
                        output.lines.push(line);
                    }
                    if output.auto_scroll {
                        output.snap_to_bottom_on_render = true;
                    }
                }
            }
            Ok(BackgroundEvent::Complete(done)) => {
                outcome = Some(done);
                break;
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                app.background_action = None;
                logger::tui_background_action_disconnected();
                return;
            }
        }
    }

    let Some(outcome) = outcome else {
        return;
    };

    app.background_action = None;
    logger::tui_background_action_complete(&outcome.title, &outcome.status);

    let keep_existing_lines = outcome.append_lines;
    let lines = outcome.lines;
    if let Some(Modal::Output(output)) = app.modal.as_mut() {
        let was_auto_scroll = output.auto_scroll;
        output.title = outcome.title;
        output.status = outcome.status;
        if keep_existing_lines {
            output.lines.extend(lines);
        } else {
            output.lines = lines;
        }
        output.running = false;
        output.auto_scroll = false;
        if was_auto_scroll {
            output.snap_to_bottom_on_render = true;
        }
        output.follow_up = outcome.follow_up;
    } else {
        app.modal = Some(Modal::Output(OutputState {
            title: outcome.title,
            status: outcome.status,
            lines,
            scroll: 0,
            max_scroll: 0,
            page_step: 10,
            auto_scroll: false,
            snap_to_bottom_on_render: false,
            running: false,
            follow_up: outcome.follow_up,
        }));
    }

    if let Screen::Tools(state) = &mut app.screen {
        state.refresh_fast(app.install_root.as_deref());
    }
    start_bg_status_check(app);
}

pub(crate) fn poll_transition(app: &mut App) {
    let Some(transition) = app.transition.as_mut() else {
        return;
    };
    transition.step = transition.step.saturating_add(1);
    if transition.step >= transition.steps {
        finish_transition(app);
    }
}

pub(crate) fn finish_transition(app: &mut App) {
    app.transition = None;
    if app.pending_status_refresh {
        app.pending_status_refresh = false;
        start_bg_status_check(app);
    }
}
