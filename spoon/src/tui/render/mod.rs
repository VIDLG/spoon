mod render_modals;
mod render_pages;
mod render_shared;

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::widgets::{Clear, Widget};

use self::render_modals::render_modal;
use self::render_pages::render_screen;
use self::render_shared::transition_progress;

use super::{App, AppConfigSnapshot, TransitionDirection};

struct PageRenderContext<'a> {
    transient_hint: Option<&'a str>,
    statuses_snapshot: &'a [super::ToolStatus],
    config_snapshot: &'a AppConfigSnapshot,
}

pub(super) fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    app.last_frame_area = area;
    let buf = frame.buffer_mut();
    Clear.render(area, buf);
    let ctx = PageRenderContext {
        transient_hint: app.status_hint.as_deref(),
        statuses_snapshot: &app.statuses_snapshot,
        config_snapshot: &app.config_snapshot,
    };

    if let Some(transition) = app.transition.as_mut() {
        let from = transition.from.clone();
        render_transition(buf, area, &from, &app.screen, transition, &ctx);
    } else {
        render_screen(
            buf,
            &mut app.screen,
            area,
            ctx.transient_hint,
            ctx.statuses_snapshot,
            ctx.config_snapshot,
        );
    }

    if let Some(modal) = app.modal.as_mut() {
        render_modal(buf, area, &app.screen, modal, app.status_hint.as_deref());
    }
}

fn render_transition(
    buf: &mut Buffer,
    area: ratatui::layout::Rect,
    from: &super::Screen,
    to: &super::Screen,
    transition: &mut super::Transition,
    ctx: &PageRenderContext<'_>,
) {
    let local = ratatui::layout::Rect::new(0, 0, area.width, area.height);
    if transition
        .cache
        .as_ref()
        .is_none_or(|cache| cache.area != local)
    {
        let mut from_buf = Buffer::empty(local);
        let mut to_buf = Buffer::empty(local);
        render_screen(
            &mut from_buf,
            &mut from.clone(),
            local,
            ctx.transient_hint,
            ctx.statuses_snapshot,
            ctx.config_snapshot,
        );
        render_screen(
            &mut to_buf,
            &mut to.clone(),
            local,
            ctx.transient_hint,
            ctx.statuses_snapshot,
            ctx.config_snapshot,
        );
        transition.cache = Some(super::TransitionCache {
            area: local,
            from_buf,
            to_buf,
        });
    }

    let Some(cache) = transition.cache.as_ref() else {
        return;
    };

    let progress = transition_progress(area.width, transition.step, transition.steps.max(1)) as i32;
    let width = area.width as i32;
    let (from_offset, to_offset) = match transition.direction {
        TransitionDirection::Forward => (-progress, width - progress),
        TransitionDirection::Backward => (progress, -(width - progress)),
    };

    blit_buffer(buf, area, &cache.from_buf, from_offset);
    blit_buffer(buf, area, &cache.to_buf, to_offset);
}

fn blit_buffer(
    target: &mut Buffer,
    target_area: ratatui::layout::Rect,
    source: &Buffer,
    x_offset: i32,
) {
    for y in 0..source.area.height {
        for x in 0..source.area.width {
            let dest_x = x as i32 + x_offset;
            if dest_x < 0 || dest_x >= target_area.width as i32 {
                continue;
            }
            let src = source[(x, y)].clone();
            target[(target_area.x + dest_x as u16, target_area.y + y)] = src;
        }
    }
}
