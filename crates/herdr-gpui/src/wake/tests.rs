#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use futures_lite::future::{block_on, ready};
use std::{sync::mpsc, thread, time::Duration};

/// Runs `each` on its own thread, so a lost wakeup fails the test instead of
/// hanging the suite.
fn handled(event: &'static Event, mut handle: impl FnMut() -> bool + Send + 'static) -> usize {
    let (done, finished) = mpsc::channel();
    thread::spawn(move || {
        let mut calls = 0;
        block_on(each(
            event,
            || {
                calls += 1;
                handle()
            },
            || ready(()),
        ));
        let _ = done.send(calls);
    });
    finished
        .recv_timeout(Duration::from_secs(5))
        .expect("an update was never handled")
}

#[test]
fn news_during_handling_wakes_the_next_round() {
    static EVENT: Event = Event::new();
    let mut round = 0;
    // The first round sees news arrive while it runs; nothing notifies after.
    let calls = handled(&EVENT, move || {
        round += 1;
        if round == 1 {
            EVENT.notify(usize::MAX);
        }
        round < 2
    });
    assert_eq!(calls, 2);
}

/// The pause follows a round rather than the news after it, so an update
/// after a quiet spell is handled as soon as it lands.
#[test]
fn the_pause_comes_before_waiting_for_news() {
    static EVENT: Event = Event::new();
    let (paused_tx, paused_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    thread::spawn(move || {
        let mut round = 0;
        block_on(each(
            &EVENT,
            || {
                round += 1;
                round < 2
            },
            || {
                let _ = paused_tx.send(());
                ready(())
            },
        ));
        let _ = done_tx.send(round);
    });
    // News is sent only once the first round has paused; had the pause
    // waited for that news instead, it would never come.
    paused_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("no pause after the first round");
    EVENT.notify(usize::MAX);
    let rounds = done_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("the news was never handled");
    assert_eq!(rounds, 2);
}
