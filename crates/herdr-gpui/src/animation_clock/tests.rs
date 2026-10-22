#![allow(clippy::expect_used)]

// Not `super::*`: GPUI's glob brings its own `test` attribute, which would
// shadow the standard one.
use super::{Pending, TICK, looping, phase, steps, until_tick};
use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement, Render, Styled, TestAppContext,
    VisualTestContext, Window, div, px,
};
use std::{cell::RefCell, rc::Rc, time::Duration};

#[test]
fn every_loop_is_measured_on_one_grid() {
    let period = Duration::from_millis(400);
    assert_eq!(phase(period, Duration::ZERO), 0.);
    assert!((phase(period, Duration::from_millis(100)) - 0.25).abs() < 1e-6);
    assert!(phase(period, period).abs() < 1e-6);
    // Two loops drawn within one tick wake on the same tick.
    let early = Duration::from_millis(1010);
    let late = Duration::from_millis(1090);
    assert_eq!(early + until_tick(early), late + until_tick(late));
    assert_eq!(until_tick(TICK), TICK);
    // A stepped loop moves one frame a tick.
    let frames = 10;
    let frame = |ticks: u32| (phase(steps(frames), TICK * ticks) * frames as f32).floor();
    assert_eq!(
        (0..12).map(frame).collect::<Vec<_>>(),
        [0., 1., 2., 3., 4., 5., 6., 7., 8., 9., 0., 1.]
    );
}

/// A view drawing `loops` looping elements, and maybe a child view of its own.
struct Loops {
    loops: usize,
    renders: usize,
    child: Option<Entity<Loops>>,
    phases: Rc<RefCell<Vec<f32>>>,
}

impl Render for Loops {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.renders += 1;
        let phases = self.phases.clone();
        div()
            .size_full()
            .children((0..self.loops).map(move |_| {
                let phases = phases.clone();
                looping(div().size(px(4.)), steps(4), move |dot, delta| {
                    phases.borrow_mut().push(delta);
                    dot.opacity(delta)
                })
            }))
            .children(self.child.clone())
    }
}

/// The views the window draws, and the views waiting for the next tick.
fn pending(cx: &mut VisualTestContext) -> (bool, usize) {
    cx.update(|_, cx| {
        let pending = cx.global::<Pending>();
        (pending.armed, pending.views.len())
    })
}

fn renders(views: &[&Entity<Loops>], cx: &mut VisualTestContext) -> Vec<usize> {
    cx.update(|_, cx| views.iter().map(|view| view.read(cx).renders).collect())
}

#[gpui::test]
fn loops_in_two_views_redraw_together_once_a_tick(cx: &mut TestAppContext) {
    let (parent, cx) = cx.add_window_view(|_, cx| {
        let child = cx.new(|_| Loops {
            loops: 1,
            renders: 0,
            child: None,
            phases: Rc::default(),
        });
        Loops {
            loops: 3,
            renders: 0,
            child: Some(child),
            phases: Rc::default(),
        }
    });
    let child = cx.update(|_, cx| parent.read(cx).child.clone().expect("a child view"));
    cx.run_until_parked();
    // Four loops in two views ask for one tick, once per view.
    assert_eq!(pending(cx), (true, 2));
    let before = renders(&[&parent, &child], cx);
    for round in 1..=3 {
        // Only the tick redraws: the window draws what it notified.
        cx.executor().advance_clock(TICK);
        cx.run_until_parked();
        let now = renders(&[&parent, &child], cx);
        assert_eq!(now, vec![before[0] + round, before[1] + round]);
        assert_eq!(pending(cx), (true, 2));
    }
    // The loops moved with the clock, a quarter of the period a tick.
    let phases = cx.update(|_, cx| parent.read(cx).phases.borrow().clone());
    let mut seen: Vec<_> = phases
        .iter()
        .map(|delta| (delta * 4.).floor() as u8)
        .collect();
    seen.dedup();
    assert!(seen.len() >= 3, "phases did not advance: {phases:?}");
    // With its loops gone, a view stops asking for ticks.
    cx.update(|_, cx| {
        parent.update(cx, |view, cx| {
            view.loops = 0;
            view.child = None;
            cx.notify();
        })
    });
    cx.run_until_parked();
    cx.executor().advance_clock(TICK);
    cx.run_until_parked();
    assert_eq!(pending(cx), (false, 0));
}
