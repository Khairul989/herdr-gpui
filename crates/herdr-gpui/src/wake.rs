//! Wakes windows when a connection has news, so an update is handled when
//! it arrives instead of on the next poll or display frame.
use event_listener::Event;
use std::future::Future;

static UPDATES: Event = Event::new();

/// Called after a connection's inbox changes, from any thread.
pub(crate) fn notify() {
    UPDATES.notify(usize::MAX);
}

/// Calls `handle` now and after every [`notify`], until it returns false.
/// `pause` runs after each call, so a stream of updates is handled at most
/// once per pause rather than once per event, while news after a quiet spell
/// is handled as soon as it lands.
pub(crate) async fn each_update<F: Future<Output = ()>>(
    handle: impl FnMut() -> bool,
    pause: impl FnMut() -> F,
) {
    each(&UPDATES, handle, pause).await;
}

async fn each<F: Future<Output = ()>>(
    event: &Event,
    mut handle: impl FnMut() -> bool,
    mut pause: impl FnMut() -> F,
) {
    loop {
        // Listen before handling: news that lands while `handle` runs, or
        // during the pause after it, still wakes the next round.
        let next = event.listen();
        if !handle() {
            return;
        }
        pause().await;
        next.await;
    }
}

#[cfg(test)]
mod tests;
