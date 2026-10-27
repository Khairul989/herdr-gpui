use super::*;
use crate::sidebar::{
    AgentGroupCounts,
    counts::{focus, shown_counts},
};

fn counts(
    blocked: usize,
    working: usize,
    done: usize,
    idle: usize,
    unknown: usize,
) -> AgentGroupCounts {
    AgentGroupCounts {
        blocked,
        done,
        idle,
        unknown,
        working,
    }
}

#[test]
fn shown_counts_orders_buckets_by_urgency_and_drops_empty_ones() {
    assert_eq!(
        shown_counts(&counts(1, 2, 3, 4, 5)),
        [
            (AgentStatus::Blocked, 1),
            (AgentStatus::Working, 2),
            (AgentStatus::Done, 3),
            (AgentStatus::Idle, 4),
            (AgentStatus::Unknown, 5),
        ]
    );
    assert_eq!(
        shown_counts(&counts(0, 2, 0, 0, 5)),
        [(AgentStatus::Working, 2), (AgentStatus::Unknown, 5)]
    );
    // A group where everyone is idle shows only its idle count.
    assert_eq!(
        shown_counts(&counts(0, 0, 0, 3, 0)),
        [(AgentStatus::Idle, 3)]
    );
    assert!(shown_counts(&counts(0, 0, 0, 0, 0)).is_empty());
}

#[test]
fn focus_names_the_most_urgent_live_bucket_before_any_quiet_one() {
    assert_eq!(
        focus(&counts(1, 2, 3, 4, 5)),
        Some((AgentStatus::Blocked, 1))
    );
    assert_eq!(
        focus(&counts(0, 2, 3, 4, 5)),
        Some((AgentStatus::Working, 2))
    );
    assert_eq!(focus(&counts(0, 0, 3, 4, 5)), Some((AgentStatus::Done, 3)));
    // Nothing live: the first non-empty quiet bucket names the pill.
    assert_eq!(focus(&counts(0, 0, 0, 4, 5)), Some((AgentStatus::Idle, 4)));
    assert_eq!(
        focus(&counts(0, 0, 0, 0, 5)),
        Some((AgentStatus::Unknown, 5))
    );
    assert_eq!(focus(&counts(0, 0, 0, 0, 0)), None);
}
