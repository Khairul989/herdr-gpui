//! One clock for every looping animation. A looping element asks for its view
//! to be redrawn at the next shared tick, so any number of spinners cost one
//! redraw per tick, at the rate the herdr TUI animates its own, instead of a
//! timer each that fires out of step with the others.
use gpui::{prelude::*, *};
use std::time::{Duration, Instant};

/// Ten redraws a second: the herdr TUI's spinner rate.
pub(crate) const TICK: Duration = Duration::from_millis(100);

/// The period of a loop that steps through `frames` frames, one a tick, so
/// none is skipped and none held for two.
pub(crate) fn steps(frames: usize) -> Duration {
    TICK * u32::try_from(frames).unwrap_or(u32::MAX)
}

/// How far through `period` the clock is at `elapsed` since its epoch, from
/// 0 up to 1.
fn phase(period: Duration, elapsed: Duration) -> f32 {
    let period = period.as_nanos().max(1);
    (elapsed.as_nanos() % period) as f32 / period as f32
}

/// The wait from `elapsed` since the epoch until the next shared tick.
fn until_tick(elapsed: Duration) -> Duration {
    let tick = TICK.as_nanos();
    Duration::from_nanos((tick - elapsed.as_nanos() % tick) as u64)
}

/// Views that drew a loop since the last tick, whether a tick is due, and
/// where every loop's phase is measured from, so loops of one period stay in
/// step wherever they are drawn.
#[derive(Default)]
struct Pending {
    views: Vec<EntityId>,
    armed: bool,
    epoch: Option<Instant>,
}

impl Global for Pending {}

/// Time on GPUI's clock since the shared epoch, which starts at first use.
fn elapsed(cx: &mut App) -> Duration {
    let now = cx.background_executor().now();
    let epoch = *cx.default_global::<Pending>().epoch.get_or_insert(now);
    now.saturating_duration_since(epoch)
}

fn redraw_at_next_tick(view: EntityId, cx: &mut App) {
    let wait = until_tick(elapsed(cx));
    let pending = cx.default_global::<Pending>();
    if !pending.views.contains(&view) {
        pending.views.push(view);
    }
    if pending.armed {
        return;
    }
    pending.armed = true;
    let tick = cx.background_executor().timer(wait);
    cx.spawn(async move |cx| {
        tick.await;
        cx.update(|cx| {
            let pending = cx.default_global::<Pending>();
            pending.armed = false;
            for view in std::mem::take(&mut pending.views) {
                cx.notify(view);
            }
        });
    })
    .detach();
}

/// `element` animated by `animator` with the shared clock's phase through
/// `period`, from 0 up to 1, forever. Reduced motion holds it at 0.
pub(crate) fn looping<E: IntoElement + 'static>(
    element: E,
    period: Duration,
    animator: impl FnOnce(E, f32) -> E + 'static,
) -> Looping<E> {
    Looping {
        parts: Some((element, Box::new(animator))),
        period,
    }
}

/// Builds a frame of the loop from the element and the clock's phase.
type Animator<E> = Box<dyn FnOnce(E, f32) -> E>;

pub(crate) struct Looping<E> {
    parts: Option<(E, Animator<E>)>,
    period: Duration,
}

impl<E: IntoElement + 'static> IntoElement for Looping<E> {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl<E: IntoElement + 'static> Element for Looping<E> {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, AnyElement) {
        let mut element = match self.parts.take() {
            Some((element, animator)) => {
                let delta = if cx.reduce_motion() {
                    0.
                } else {
                    redraw_at_next_tick(window.current_view(), cx);
                    // QA > Slow Animations stretches the loop like GPUI's own.
                    phase(crate::motion::duration(self.period), elapsed(cx))
                };
                animator(element, delta).into_any_element()
            }
            // An element is laid out once; a second call has nothing to draw.
            None => Empty.into_any_element(),
        };
        (element.request_layout(window, cx), element)
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        element: &mut AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        element: &mut AnyElement,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx);
    }
}

#[cfg(test)]
mod tests;
