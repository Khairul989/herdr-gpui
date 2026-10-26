use super::*;
use crate::sidebar::agents::{AgentGroupCounts, workspace_agent_groups};

#[test]
fn agent_groups_count_distinct_status_buckets_and_describe_every_bucket() {
    let counts = AgentGroupCounts::from_statuses(
        [
            (AgentStatus::Idle, 1),
            (AgentStatus::Blocked, 2),
            (AgentStatus::Done, 3),
            (AgentStatus::Working, 4),
            (AgentStatus::Unknown, 5),
        ]
        .into_iter()
        .flat_map(|(status, count)| std::iter::repeat_n(status, count)),
    );
    assert_eq!(
        counts,
        AgentGroupCounts {
            blocked: 2,
            done: 3,
            idle: 1,
            unknown: 5,
            working: 4
        }
    );
    assert_eq!(counts.total(), 15);
    assert_eq!(
        counts.description(),
        "15 agents, 1 idle, 2 need attention, 3 completed, 4 working, 5 unknown"
    );
    assert_eq!(counts.compact(), ["I 1", "! 2", "✓ 3", "W 4", "? 5"]);
    let idle = AgentGroupCounts::from_statuses([AgentStatus::Idle]);
    assert_eq!(idle.compact(), ["I 1", "! 0", "✓ 0"]);
    assert_eq!(AgentGroupCounts::from_statuses([]).total(), 0);
}

fn key(endpoint: &str, boot: &str, workspace: &str) -> (String, String, String) {
    (endpoint.into(), boot.into(), workspace.into())
}

#[test]
fn agent_groups_keep_first_member_and_member_order() {
    let a = key("local", "session-a", "w0");
    let b = key("local", "session-a", "w1");
    let c = key("local", "session-a", "w2");
    assert_eq!(
        workspace_agent_groups(&[a.clone(), b.clone(), a, c, b], false),
        [vec![0, 2], vec![1, 4], vec![3]]
    );
    assert!(workspace_agent_groups(&[], false).is_empty());
    assert_eq!(
        workspace_agent_groups(&[key("local", "boot", "w0")], false),
        [vec![0]]
    );
}

#[test]
fn agent_groups_fence_endpoint_boot_and_workspace_identity() {
    let keys = [
        key("local", "session-a", "w0"),
        key("ssh:remote", "session-a", "w0"),
        key("local", "session-b", "w0"),
        key("local", "session-a", "w1"),
        key("local", "session-a", "w0"),
        key("a-b", "c", "w0"),
        key("a", "b-c", "w0"),
    ];
    assert_eq!(
        workspace_agent_groups(&keys, false),
        [vec![0, 4], vec![1], vec![2], vec![3], vec![5], vec![6]]
    );
}

#[test]
fn agent_groups_flat_views_keep_interleaved_order_and_empty_membership() {
    let a = key("local", "boot", "w0");
    let b = key("local", "boot", "w1");
    assert_eq!(
        workspace_agent_groups(&[a.clone(), b, a], true),
        [vec![0], vec![1], vec![2]]
    );
    assert!(workspace_agent_groups(&[], true).is_empty());
}
