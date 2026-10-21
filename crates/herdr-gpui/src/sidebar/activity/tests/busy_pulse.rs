//! Overview's bar icon pulses while agents work. The pulse runs in the
//! window's mark layer, so time passing redraws the layer and never rebuilds
//! the cached sidebar the bar sits in.
#![allow(clippy::expect_used)]

use crate::{
    HerdrWindow,
    mark_layer::Marks,
    sidebar::layout_tests::{fixture_window, full_draw},
    state::ConnectionStatus,
};
use gpui::{Entity, VisualTestContext, px, size};
use std::time::Duration;

/// Draws the whole window with the bar shown or not, and counts the marks
/// the layer then holds.
fn layered_marks(view: &Entity<HerdrWindow>, bar: bool, cx: &mut VisualTestContext) -> usize {
    cx.update(|window, cx| {
        view.update(cx, |view, _| view.config.sidebar_style.activity_bar = bar);
        full_draw(window, cx).clear(cx);
        Marks::of(window, cx).expect("the window has a layer").len()
    })
}

#[gpui::test]
fn a_busy_overview_icon_pulses_without_rebuilding_the_sidebar(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.status = ConnectionStatus::Connected;
        view
    });
    cx.simulate_resize(size(px(1000.), px(700.)));
    cx.update(|_, cx| view.update(cx, |view, cx| view.poll_overview(cx)));
    let (sidebar, layer) = cx.update(|_, cx| {
        let view = view.read(cx);
        assert!(view.overview_working() > 0, "the fixture's agents work");
        (view.sidebar_view.clone(), view.mark_layer.clone())
    });
    // The agents' own spinners are in the layer either way; the busy icon
    // is the one mark the bar adds.
    let without_bar = layered_marks(&view, false, cx);
    assert_eq!(layered_marks(&view, true, cx), without_bar + 1);
    assert!(cx.debug_bounds("activity-overview-glyph").is_some());
    let (renders, layer_renders) =
        cx.update(|_, cx| (sidebar.read(cx).renders, layer.read(cx).renders));
    for _ in 0..4 {
        cx.executor().advance_clock(Duration::from_millis(150));
        cx.update(|window, cx| {
            window.simulate_next_frame(cx);
        });
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }
    cx.update(|_, cx| {
        assert_eq!(sidebar.read(cx).renders, renders);
        assert!(layer.read(cx).renders >= layer_renders + 4);
    });
    assert!(cx.debug_bounds("activity-overview-glyph").is_some());
}
