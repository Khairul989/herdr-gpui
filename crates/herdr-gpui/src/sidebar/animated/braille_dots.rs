//! Herdr's dots with the terminal client's braille spinner: a hollow ring
//! when idle, a filled disc when done, a small dot when unknown, the braille
//! cell while working, and a filled disc with a cross cut out of it when an
//! agent waits on you. Only the spinner moves.

use super::{BRAILLE, STATUS_DOT_UNKNOWN, braille, disc, dot_diameter};
use gpui::{prelude::*, *};
use herdr_client::protocol::AgentStatus;

pub(super) fn mark(status: AgentStatus, size: f32, color: u32) -> AnyElement {
    let dot = cell_height(size);
    match status {
        AgentStatus::Working => braille("status-braille", &BRAILLE, size, color).into_any_element(),
        // The asset's disc has a radius of 7 in its 16-unit box, so the box
        // is drawn larger to give the disc the other dots' diameter.
        AgentStatus::Blocked => svg()
            .debug_selector(|| "status-braille-dots-blocked".into())
            .path("icons/status-solid-blocked.svg")
            .size(px((dot * 16. / 14.).round()))
            .flex_none()
            .text_color(rgb(color))
            .into_any_element(),
        AgentStatus::Done => disc(dot, color)
            .debug_selector(|| "status-braille-dots-done".into())
            .into_any_element(),
        AgentStatus::Idle => div()
            .debug_selector(|| "status-braille-dots-idle".into())
            .size(px(dot))
            .flex_none()
            .rounded_full()
            .border_1()
            .border_color(rgb(color))
            .into_any_element(),
        AgentStatus::Unknown => disc(STATUS_DOT_UNKNOWN, color)
            .debug_selector(|| "status-braille-dots-unknown".into())
            .into_any_element(),
    }
}

/// The height of the spinner's three rows of dots in a `size` slot, so the
/// settled dots stand as tall as the spinner beside them.
pub(super) fn cell_height(size: f32) -> f32 {
    dot_diameter(size) * 3. + 2.
}

#[cfg(test)]
mod tests;
