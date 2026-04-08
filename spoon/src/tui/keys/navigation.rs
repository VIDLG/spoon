use crossterm::event::KeyCode;

use super::super::{
    App, Screen, TransitionDirection, request_bg_status_check, start_page_transition,
};

pub(super) fn handle_top_navigation(app: &mut App, code: KeyCode) -> bool {
    let current = super::super::top_page_index(&app.screen);
    let nav = match code {
        KeyCode::Left | KeyCode::Char('h') if current > 0 => {
            Some((current - 1, TransitionDirection::Backward))
        }
        KeyCode::Right | KeyCode::Char('l') if current < 1 => {
            Some((current + 1, TransitionDirection::Forward))
        }
        KeyCode::Char('1') if current != 0 => Some((0, page_direction(current, 0))),
        KeyCode::Char('2') if current != 1 => Some((1, page_direction(current, 1))),
        _ => None,
    };

    let Some((target_index, direction)) = nav else {
        return false;
    };
    let current_screen = app.screen.clone();
    super::super::remember_screen(app, &current_screen);
    let next = super::super::cached_screen_for_top_page(
        app,
        target_index,
        app.install_root.as_deref(),
        &app.repo_root,
    );
    start_page_transition(app, next.clone(), direction);
    if matches!(next, Screen::Tools(_)) {
        request_bg_status_check(app);
    }
    true
}

pub(super) fn apply_screen_change(app: &mut App, screen: Screen) {
    let current = super::super::top_page_index(&app.screen);
    let next = super::super::top_page_index(&screen);
    let current_screen = app.screen.clone();
    super::super::remember_screen(app, &current_screen);
    if current != next {
        start_page_transition(app, screen.clone(), page_direction(current, next));
    } else {
        app.screen = screen.clone();
    }
    if matches!(screen, Screen::Tools(_)) {
        request_bg_status_check(app);
    }
}

fn page_direction(current: usize, target: usize) -> TransitionDirection {
    if (current + 1) % 2 == target {
        TransitionDirection::Forward
    } else if (target + 1) % 2 == current {
        TransitionDirection::Backward
    } else if target > current {
        TransitionDirection::Forward
    } else {
        TransitionDirection::Backward
    }
}
