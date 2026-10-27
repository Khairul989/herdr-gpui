#![allow(clippy::unwrap_used)]
use super::*;
use crate::config::StatusIcons;
use core::prelude::v1::test;
use gpui::TestAppContext;

struct SizedMark {
    status: AgentStatus,
    size: f32,
}

impl Render for SizedMark {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(super::super::mark(
            StatusIcons::BrailleDots,
            self.status,
            self.size,
            0xf9e2af,
        ))
    }
}

fn drawn(
    status: AgentStatus,
    side: f32,
    selector: &'static str,
    cx: &mut TestAppContext,
) -> Bounds<Pixels> {
    let (_, cx) = cx.add_window_view(|_, _| SizedMark { status, size: side });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.debug_bounds(selector)
        .unwrap_or_else(|| panic!("{status:?} at {side} drew no {selector}"))
}

/// The set reads as the terminal client's: braille while working, Herdr's
/// dots once settled, and a crossed disc when blocked, each its own mark.
#[gpui::test]
fn each_status_draws_its_own_mark(cx: &mut TestAppContext) {
    for (status, selector) in [
        (AgentStatus::Working, "status-braille"),
        (AgentStatus::Blocked, "status-braille-dots-blocked"),
        (AgentStatus::Done, "status-braille-dots-done"),
        (AgentStatus::Idle, "status-braille-dots-idle"),
        (AgentStatus::Unknown, "status-braille-dots-unknown"),
    ] {
        drawn(status, 16., selector, cx);
    }
}

/// Done, idle, and blocked dots stand as tall as the spinner's cell beside
/// them, and an unknown agent keeps Herdr's small dot.
#[gpui::test]
fn settled_dots_match_the_spinner_height(cx: &mut TestAppContext) {
    for side in [10., 14., 16., 20., 28.] {
        let spinner = drawn(AgentStatus::Working, side, "status-braille-dots", cx);
        let height = spinner.size.height;
        assert_eq!(height, px(cell_height(side)), "spinner at {side}");
        for (status, selector) in [
            (AgentStatus::Done, "status-braille-dots-done"),
            (AgentStatus::Idle, "status-braille-dots-idle"),
        ] {
            let dot = drawn(status, side, selector, cx);
            assert_eq!(dot.size, size(height, height), "{status:?} at {side}");
        }
        // The asset's disc fills 14 of its 16 units.
        let blocked = drawn(
            AgentStatus::Blocked,
            side,
            "status-braille-dots-blocked",
            cx,
        );
        let disc = blocked.size.height * (14. / 16.);
        assert!(
            (disc - height).abs() <= px(1.),
            "blocked at {side}: {blocked:?}"
        );
        let unknown = drawn(
            AgentStatus::Unknown,
            side,
            "status-braille-dots-unknown",
            cx,
        );
        assert_eq!(
            unknown.size.height,
            px(STATUS_DOT_UNKNOWN),
            "unknown at {side}"
        );
    }
}
