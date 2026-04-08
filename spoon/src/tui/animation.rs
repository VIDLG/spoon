pub(crate) const IDLE_POLL_MS: u64 = 50;
pub(crate) const ANIMATION_POLL_MS: u64 = 24;

pub(crate) const PAGE_TRANSITION_STEPS: u16 = 9;

pub(crate) fn smoothstep(step: u16, steps: u16) -> f32 {
    if steps == 0 {
        return 1.0;
    }
    let t = (step.min(steps) as f32) / (steps as f32);
    t * t * (3.0 - 2.0 * t)
}
