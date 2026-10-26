use super::*;

fn group_snapshot(statuses: impl IntoIterator<Item = AgentStatus>) -> ClientShellSnapshot {
    let mut listing = snapshot(2);
    let template = listing.agents[0].clone();
    listing.agents = statuses
        .into_iter()
        .enumerate()
        .map(|(index, agent_status)| ClientShellAgent {
            pane_id: format!("p{index}"),
            agent_status,
            ..template.clone()
        })
        .collect();
    listing
}

#[gpui::test]
fn agent_groups_live_counts_survive_collapse_and_narrow_width(cx: &mut gpui::TestAppContext) {
    let counts = [
        (AgentStatus::Idle, 1),
        (AgentStatus::Blocked, 2),
        (AgentStatus::Done, 3),
        (AgentStatus::Working, 4),
        (AgentStatus::Unknown, 5),
    ];
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.snapshot =
            Some(Arc::new(group_snapshot(counts.into_iter().flat_map(
                |(status, count)| std::iter::repeat_n(status, count),
            ))));
        view.sidebar_width = Some(160.);
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.run_until_parked();
    cx.update(|window, cx| {
        cx.default_global::<TextProbes>().0.clear();
        full_draw(window, cx).clear(cx);
        let probes = &cx.global::<TextProbes>().0;
        for text in ["I 1", "! 2", "✓ 3", "W 4", "? 5"] {
            assert!(
                probes.contains_key(text),
                "missing live group count: {text}"
            );
        }
    });
    let header = cx.debug_bounds("agent-group-local-layout-test-w0").unwrap();
    cx.simulate_click(header.center(), Default::default());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let listing = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
            listing.agents[1].agent_status = AgentStatus::Idle;
            listing.agents[6].agent_status = AgentStatus::Done;
            cx.notify();
        });
        cx.default_global::<TextProbes>().0.clear();
        full_draw(window, cx).clear(cx);
        let probes = &cx.global::<TextProbes>().0;
        let sidebar = probes.get("I 2").unwrap().0;
        for text in ["I 2", "! 1", "✓ 4", "W 3", "? 5"] {
            let (bounds, rendered, glyphs) = probes.get(text).unwrap();
            assert_eq!(rendered, text);
            assert!(*glyphs <= bounds.size.width, "{text} clipped: {bounds:?}");
            assert!(bounds.left() >= sidebar.left());
        }
    });
    let sidebar = cx.debug_bounds("sidebar").unwrap();
    cx.update(|_, cx| {
        for text in ["I 2", "! 1", "✓ 4", "W 3", "? 5"] {
            let bounds = cx.global::<TextProbes>().0[text].0;
            assert!(
                bounds.right() <= sidebar.right(),
                "{text} past narrow sidebar: {bounds:?}"
            );
        }
    });
    assert!(cx.debug_bounds("row-agent-p0").is_none());
}

#[gpui::test]
fn agent_groups_threshold_transitions_keep_singletons_visible(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut listing = group_snapshot([AgentStatus::Idle]);
        listing.agents[0].focused = true;
        view.live.snapshot = Some(Arc::new(listing));
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.run_until_parked();
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    let group = "agent-group-local-layout-test-w0";
    assert!(cx.debug_bounds(group).is_none());
    assert!(cx.debug_bounds("row-agent-p0").is_some());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let listing = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
            let mut second = listing.agents[0].clone();
            second.pane_id = "p1".into();
            second.focused = false;
            second.agent_status = AgentStatus::Unknown;
            listing.agents.push(second);
            cx.notify();
        });
        full_draw(window, cx).clear(cx);
    });
    let header = cx.debug_bounds(group).unwrap();
    assert!(cx.debug_bounds("row-agent-p1").is_some());
    cx.simulate_click(header.center(), Default::default());
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("row-agent-p0").is_none());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            Arc::make_mut(view.live.snapshot.as_mut().unwrap())
                .agents
                .truncate(1);
            cx.notify();
        });
        full_draw(window, cx).clear(cx);
    });
    assert!(cx.debug_bounds(group).is_none());
    assert!(cx.debug_bounds("row-agent-p0").is_some());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let listing = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
            let mut second = listing.agents[0].clone();
            second.pane_id = "p1".into();
            second.focused = false;
            second.agent_status = AgentStatus::Done;
            listing.agents.push(second);
            cx.notify();
        });
        full_draw(window, cx).clear(cx);
    });
    assert!(cx.debug_bounds(group).is_some());
    assert!(cx.debug_bounds("row-agent-p0").is_none());
    assert!(cx.debug_bounds("row-agent-p1").is_none());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            Arc::make_mut(view.live.snapshot.as_mut().unwrap())
                .agents
                .clear();
            cx.notify();
        });
        full_draw(window, cx).clear(cx);
    });
    assert!(cx.debug_bounds(group).is_none());
    assert!(cx.debug_bounds("row-agent-p0").is_none());
}

#[gpui::test]
fn agent_groups_filter_before_zero_one_two_threshold(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut listing =
            group_snapshot([AgentStatus::Idle, AgentStatus::Unknown, AgentStatus::Done]);
        listing.workspaces.clear();
        listing.agents[1].workspace_id = "missing".into();
        listing.agents[2].workspace_id = "missing".into();
        view.live.snapshot = Some(Arc::new(listing));
        view.config.usage.inline = true;
        view.config.sidebar_layout.agents =
            toml::from_str("rows = [[\"workspace\"], [\"agent\"]]").unwrap();
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.run_until_parked();
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    let group = "agent-group-local-layout-test-w0";
    assert!(cx.debug_bounds(group).is_none());
    for row in ["row-agent-p0", "row-agent-p1", "row-agent-p2"] {
        assert!(cx.debug_bounds(row).is_none());
    }
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            Arc::make_mut(view.live.snapshot.as_mut().unwrap()).workspaces = snapshot(2).workspaces;
            cx.notify();
        });
        full_draw(window, cx).clear(cx);
    });
    assert!(cx.debug_bounds(group).is_none());
    assert!(cx.debug_bounds("row-agent-p0").is_some());
    assert!(cx.debug_bounds("row-agent-p1").is_none());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            Arc::make_mut(view.live.snapshot.as_mut().unwrap()).agents[1].workspace_id =
                "w0".into();
            cx.notify();
        });
        cx.default_global::<TextProbes>().0.clear();
        full_draw(window, cx).clear(cx);
        for text in ["I 1", "! 0", "✓ 0", "? 1"] {
            assert!(cx.global::<TextProbes>().0.contains_key(text), "{text}");
        }
    });
    assert!(cx.debug_bounds(group).is_some());
    assert!(cx.debug_bounds("row-agent-p0").is_some());
    assert!(cx.debug_bounds("row-agent-p1").is_some());
    assert!(cx.debug_bounds("row-agent-p2").is_none());
}

#[gpui::test]
fn agent_groups_follow_panel_priority_and_grouped_order(cx: &mut gpui::TestAppContext) {
    use crate::preferences::AgentSort;
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut listing = group_snapshot([
            AgentStatus::Idle,
            AgentStatus::Blocked,
            AgentStatus::Working,
            AgentStatus::Done,
        ]);
        for index in [1, 3] {
            listing.agents[index].workspace_id = "w1".into();
        }
        listing.agents[3].focused = true;
        view.live.snapshot = Some(Arc::new(listing));
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.run_until_parked();
    for (sort, ids, focus) in [
        (
            AgentSort::Grouped,
            [
                "agent-group-local-layout-test-w0",
                "row-agent-p0",
                "row-agent-p2",
                "agent-group-local-layout-test-w1",
                "row-agent-p1",
                "row-agent-p3",
            ],
            5,
        ),
        (
            AgentSort::Priority,
            [
                "agent-group-local-layout-test-w1",
                "row-agent-p1",
                "row-agent-p3",
                "agent-group-local-layout-test-w0",
                "row-agent-p2",
                "row-agent-p0",
            ],
            2,
        ),
    ] {
        view.update(cx, |view, cx| {
            view.agent_sort = sort;
            cx.notify();
        });
        cx.update(|window, cx| {
            full_draw(window, cx).clear(cx);
            full_draw(window, cx).clear(cx);
            assert_eq!(view.read(cx).sidebar_revealed[1].get(), Some(focus));
        });
        for pair in ids.windows(2) {
            let first = cx.debug_bounds(pair[0]).unwrap();
            let second = cx.debug_bounds(pair[1]).unwrap();
            assert!(first.bottom() <= second.top(), "{sort:?}: {pair:?}");
        }
    }
}

#[gpui::test]
fn agent_groups_isolate_hosts_and_session_boots_and_honor_empty_remote_views(
    cx: &mut gpui::TestAppContext,
) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.snapshot = Some(Arc::new(group_snapshot([
            AgentStatus::Idle,
            AgentStatus::Done,
        ])));
        let mut remote = crate::endpoint::Endpoint::new(
            "ssh:test".into(),
            "Remote".into(),
            ConnectTarget::Socket("/unused-agent-groups.sock".into()),
            true,
        );
        remote.live.snapshot = view.live.snapshot.clone();
        view.endpoints.push(remote);
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.run_until_parked();
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    let local = "agent-group-local-layout-test-w0";
    let remote = "agent-group-ssh:test-layout-test-w0";
    let local_header = cx.debug_bounds(local).unwrap();
    assert!(cx.debug_bounds(remote).is_some());
    cx.simulate_click(local_header.center(), Default::default());
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("agent-local-p0").is_none());
    assert!(cx.debug_bounds("agent-ssh:test-p0").is_some());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            Arc::make_mut(view.live.snapshot.as_mut().unwrap()).boot_id = "other-session".into();
            cx.notify();
        });
        full_draw(window, cx).clear(cx);
    });
    assert!(
        cx.debug_bounds("agent-group-local-other-session-w0")
            .is_some()
    );
    assert!(cx.debug_bounds("agent-local-p0").is_some());
    assert!(cx.debug_bounds("agent-ssh:test-p0").is_some());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            Arc::make_mut(view.live.snapshot.as_mut().unwrap()).boot_id = "layout-test".into();
            cx.notify();
        });
        full_draw(window, cx).clear(cx);
    });
    assert!(cx.debug_bounds(local).is_some());
    assert!(cx.debug_bounds("agent-local-p0").is_none());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let listing = Arc::make_mut(view.endpoints[1].live.snapshot.as_mut().unwrap());
            listing.agent_view_label = Some("empty remote view".into());
            listing.agent_order.clear();
            cx.notify();
        });
        full_draw(window, cx).clear(cx);
    });
    assert!(cx.debug_bounds(local).is_none());
    assert!(cx.debug_bounds(remote).is_none());
    assert!(cx.debug_bounds("agent-local-p0").is_some());
    assert!(cx.debug_bounds("agent-local-p1").is_some());
    assert!(cx.debug_bounds("agent-ssh:test-p0").is_none());
    let host = cx.debug_bounds("host-ssh:test").unwrap();
    cx.simulate_mouse_down(host.center(), gpui::MouseButton::Left, Modifiers::default());
    cx.executor().advance_clock(crate::reorder::LIFT_DELAY);
    cx.run_until_parked();
    cx.update(|window, cx| {
        full_draw(window, cx).clear(cx);
        full_draw(window, cx).clear(cx);
        assert!(view.read(cx).device_plan(false, "").is_some());
    });
    assert!(cx.debug_bounds("workspace-local-w0").is_none());
    assert!(cx.debug_bounds(local).is_none());
    assert!(cx.debug_bounds("agent-local-p0").is_some());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.cancel_device_drag(cx);
            view.endpoints[1].enabled = false;
            cx.notify();
        });
        full_draw(window, cx).clear(cx);
    });
    // A disabled host does not participate, even while its snapshot names a view.
    assert!(cx.debug_bounds(local).is_some());
    assert!(cx.debug_bounds("agent-local-p0").is_none());
}

#[gpui::test]
fn agent_groups_plugin_views_preserve_interleaved_membership(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut listing = group_snapshot([AgentStatus::Idle; 4]);
        for index in [1, 3] {
            listing.agents[index].workspace_id = "w1".into();
        }
        listing.agent_view_label = Some("interleaved".into());
        listing.agent_order = ["p2", "p1", "p0", "p3"].map(str::to_owned).into();
        view.live.snapshot = Some(Arc::new(listing));
        view.agent_sort = crate::preferences::AgentSort::Priority;
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.run_until_parked();
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    assert!(
        cx.debug_bounds("agent-group-local-layout-test-w0")
            .is_none()
    );
    assert!(
        cx.debug_bounds("agent-group-local-layout-test-w1")
            .is_none()
    );
    for pair in [
        "row-agent-p2",
        "row-agent-p1",
        "row-agent-p0",
        "row-agent-p3",
    ]
    .windows(2)
    {
        let first = cx.debug_bounds(pair[0]).unwrap();
        let second = cx.debug_bounds(pair[1]).unwrap();
        assert!(first.bottom() <= second.top(), "{pair:?}");
    }
}

#[gpui::test]
fn agent_groups_reveal_collapsed_focused_header_with_visible_child_indices(
    cx: &mut gpui::TestAppContext,
) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut listing = group_snapshot([AgentStatus::Idle; 24]);
        listing.workspaces = snapshot(12).workspaces;
        for (index, agent) in listing.agents.iter_mut().enumerate() {
            agent.workspace_id = format!("w{}", index / 2);
            agent.focused = index == 23;
        }
        listing.focused_pane_id = Some("p23".into());
        view.live.snapshot = Some(Arc::new(listing));
        view
    });
    cx.simulate_resize(size(px(800.), px(712.)));
    cx.run_until_parked();
    cx.update(|window, cx| {
        full_draw(window, cx).clear(cx);
        full_draw(window, cx).clear(cx);
        assert_eq!(view.read(cx).sidebar_revealed[1].get(), Some(35));
        // Revealing the last child need not expose the header above it.
        view.read(cx).sidebar_scroll[1].scroll_to_item(33);
        full_draw(window, cx).clear(cx);
    });
    let header = cx
        .debug_bounds("agent-group-local-layout-test-w11")
        .unwrap();
    cx.update(|_, cx| {
        let viewport = view.read(cx).sidebar_scroll[1].bounds();
        assert!(
            viewport.contains(&header.center()),
            "header {header:?} outside {viewport:?}"
        );
    });
    cx.simulate_click(header.center(), Default::default());
    cx.update(|window, cx| {
        assert!(
            view.read(cx).collapsed_agent_groups.contains(&(
                "local".into(),
                "layout-test".into(),
                "w11".into()
            )),
            "visible header click did not collapse its group"
        );
        full_draw(window, cx).clear(cx);
        full_draw(window, cx).clear(cx);
        let view = view.read(cx);
        assert_eq!(view.sidebar_revealed[1].get(), Some(33));
        assert_eq!(
            view.live
                .snapshot
                .as_ref()
                .unwrap()
                .focused_pane_id
                .as_deref(),
            Some("p23")
        );
        let scroll = &view.sidebar_scroll[1];
        let header = scroll.bounds_for_item(33).unwrap();
        assert!(header.top() + scroll.offset().y >= scroll.bounds().top());
        assert!(header.bottom() + scroll.offset().y <= scroll.bounds().bottom());
        assert!(
            scroll.bounds_for_item(34).is_none(),
            "collapsed children must not count as scroll items"
        );
    });
    assert!(cx.debug_bounds("row-agent-p22").is_none());
    assert!(cx.debug_bounds("row-agent-p23").is_none());
}

#[gpui::test]
fn agent_groups_narrow_headers_keep_counts_below_long_labels(cx: &mut gpui::TestAppContext) {
    use crate::config::{Density, LayoutMode};
    let label = "A workspace with a deliberately long and descriptive name";
    let (_view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut listing = group_snapshot([
            AgentStatus::Idle,
            AgentStatus::Blocked,
            AgentStatus::Done,
            AgentStatus::Working,
            AgentStatus::Unknown,
        ]);
        listing.workspaces[0].label = label.into();
        view.live.snapshot = Some(Arc::new(listing));
        view.sidebar_width = Some(160.);
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.run_until_parked();
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    // Child rows repeat the workspace label; folding isolates the header's
    // glyphs in the text-keyed paint probe.
    let header = cx.debug_bounds("agent-group-local-layout-test-w0").unwrap();
    cx.simulate_click(header.center(), Default::default());
    for mode in [
        LayoutMode::from(Density::Compact),
        LayoutMode::Orca,
        LayoutMode::Superset,
    ] {
        for font in [12., 36.] {
            _view.update(cx, |view, cx| {
                view.config.layout.mode = mode;
                view.config.sidebar.size = font;
                cx.notify();
            });
            cx.update(|window, cx| {
                cx.default_global::<TextProbes>().0.clear();
                full_draw(window, cx).clear(cx);
            });
            let header = cx.debug_bounds("agent-group-local-layout-test-w0").unwrap();
            let sidebar = cx.debug_bounds("sidebar").unwrap();
            cx.update(|_, cx| {
                let probes = &cx.global::<TextProbes>().0;
                let (name, rendered, glyphs) = &probes[label];
                assert!(rendered.ends_with('…'), "{mode:?} {font}: {rendered}");
                assert!(*glyphs <= name.size.width);
                for text in ["I 1", "! 1", "✓ 1", "W 1", "? 1"] {
                    let (bounds, rendered, glyphs) = &probes[text];
                    assert_eq!(rendered, text);
                    assert!(*glyphs <= bounds.size.width);
                    assert!(
                        bounds.top() >= name.bottom(),
                        "{mode:?} {font}: count {text} {bounds:?}, name {name:?}, header {header:?}"
                    );
                    assert!(bounds.left() >= sidebar.left());
                    assert!(bounds.right() <= sidebar.right());
                    assert!(bounds.bottom() <= header.bottom());
                }
            });
        }
    }
}

#[gpui::test]
fn agent_groups_children_keep_their_pane_menu(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut listing = group_snapshot([AgentStatus::Idle, AgentStatus::Blocked]);
        listing.panes = ["p0", "p1"]
            .map(|id| {
                serde_json::from_value(serde_json::json!({
                    "pane_id": id, "workspace_id": "w0", "tab_id": "t0", "label": id,
                    "cwd": "/repo", "foreground_cwd": "/repo", "focused": false,
                    "right_click_passthrough": false
                }))
                .unwrap()
            })
            .into();
        view.live.snapshot = Some(Arc::new(listing));
        view.live.status = crate::state::ConnectionStatus::Connected;
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.run_until_parked();
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    assert!(
        cx.debug_bounds("agent-group-local-layout-test-w0")
            .is_some()
    );
    let position = cx.debug_bounds("row-agent-p1").unwrap().center();
    cx.simulate_mouse_down(position, gpui::MouseButton::Right, Modifiers::default());
    cx.update(|window, cx| {
        full_draw(window, cx).clear(cx);
        let view = view.read(cx);
        assert_eq!(view.selected_endpoint, 0);
        assert_eq!(view.menu.page, Some(crate::menu::Page::Pane));
        assert_eq!(view.pane_menu_target(), Some("p1"));
        assert!(view.menu.opening_right_click);
        assert!(view.menu.focus.is_focused(window));
    });
    cx.simulate_mouse_up(position, gpui::MouseButton::Right, Modifiers::default());
}

#[gpui::test]
fn agent_groups_do_not_change_nested_placement(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.snapshot = Some(Arc::new(group_snapshot([
            AgentStatus::Idle,
            AgentStatus::Done,
        ])));
        view.config.agents_placement = crate::config::AgentsPlacement::Nested;
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.run_until_parked();
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    assert!(
        cx.debug_bounds("agent-group-local-layout-test-w0")
            .is_none()
    );
    assert!(cx.debug_bounds("agents-section").is_none());
    assert!(cx.debug_bounds("row-agent-p0").is_some());
    assert!(cx.debug_bounds("row-agent-p1").is_some());
}

#[gpui::test]
fn agent_groups_endpoint_identity_does_not_follow_a_reused_index(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.snapshot = Some(Arc::new(group_snapshot([AgentStatus::Unknown; 2])));
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.run_until_parked();
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    let header = cx.debug_bounds("agent-group-local-layout-test-w0").unwrap();
    cx.simulate_click(header.center(), Default::default());
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("row-agent-p0").is_none());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.endpoints[0].id = "replacement".into();
            cx.notify();
        });
        full_draw(window, cx).clear(cx);
    });
    assert!(
        cx.debug_bounds("agent-group-local-layout-test-w0")
            .is_none()
    );
    assert!(
        cx.debug_bounds("agent-group-replacement-layout-test-w0")
            .is_some()
    );
    assert!(cx.debug_bounds("row-agent-p0").is_some());
    assert!(cx.debug_bounds("row-agent-p1").is_some());
}

#[gpui::test]
fn agent_groups_accessibility_items_expose_full_counts_and_expanded_state(
    cx: &mut gpui::TestAppContext,
) {
    use gpui::Element;
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.snapshot = Some(Arc::new(group_snapshot([
            AgentStatus::Idle,
            AgentStatus::Done,
        ])));
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.run_until_parked();
    for expanded in [true, false, true] {
        cx.update(|window, cx| {
            full_draw(window, cx).clear(cx);
            view.update(cx, |view, cx| {
                let counts = super::super::agents::AgentGroupCounts::from_statuses(view.live.snapshot.as_ref().unwrap().agents.iter().map(|agent| agent.agent_status));
                let row = view.agent_group_item(
                    gpui::div().id("agent-group-semantics"), format!("herdr, {}", counts.description()),
                    !expanded, ("local".into(), "layout-test".into(), "w0".into()), cx,
                );
                assert_eq!(row.a11y_role(), Some(gpui::Role::ListItem));
                let mut node = gpui::accesskit::Node::new(gpui::Role::ListItem);
                row.write_a11y_info(&mut node);
                assert_eq!(node.label(), Some("herdr, 2 agents, 1 idle, 0 need attention, 1 completed, 0 working, 0 unknown"));
                assert_eq!(node.is_expanded(), Some(expanded));
                assert_eq!(node.is_selected(), Some(!expanded));
                assert!(node.supports_action(gpui::AccessibleAction::Expand));
                assert!(node.supports_action(gpui::AccessibleAction::Collapse));
            });
        });
        let header = cx.debug_bounds("agent-group-local-layout-test-w0").unwrap();
        cx.simulate_click(header.center(), Default::default());
    }
}

#[gpui::test]
fn agent_groups_render_and_disclose_same_workspace(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let snapshot = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
        snapshot.agents[1].workspace_id = "w0".into();
        snapshot.agents[0].focused = true;
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.run_until_parked();
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    let header = cx.debug_bounds("agent-group-local-layout-test-w0");
    assert!(
        header.is_some(),
        "two listed agents in one workspace need a disclosure header"
    );
    let header = header.unwrap();
    assert!(cx.debug_bounds("row-agent-p0").is_some());
    assert!(cx.debug_bounds("row-agent-p1").is_some());
    cx.simulate_click(header.center(), Default::default());
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("row-agent-p0").is_none());
    assert!(cx.debug_bounds("row-agent-p1").is_none());
    cx.update(|_, cx| assert!(view.read(cx).live.snapshot.as_ref().unwrap().agents[0].focused));
    let disclosure = cx
        .debug_bounds("agent-group-disclosure-local-layout-test-w0")
        .unwrap();
    cx.simulate_click(disclosure.center(), Default::default());
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("row-agent-p0").is_some());
}

#[gpui::test]
fn agent_groups_shortcuts_step_in_painted_order(cx: &mut gpui::TestAppContext) {
    use crate::{controls::Command, sidebar::order::step};
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut listing = group_snapshot([AgentStatus::Idle; 4]);
        for index in [1, 3] {
            listing.agents[index].workspace_id = "w1".into();
        }
        view.live.snapshot = Some(Arc::new(listing));
        view.agent_sort = crate::preferences::AgentSort::Grouped;
        view
    });
    let panes = |view: &HerdrWindow| {
        view.sidebar_agents()
            .into_iter()
            .map(|(_, pane)| pane.to_owned())
            .collect::<Vec<_>>()
    };
    view.update(cx, |view, _| {
        let rows = view.sidebar_agents();
        assert_eq!(panes(view), ["p0", "p2", "p1", "p3"]);
        assert_eq!(step(&rows, Some(0), 0, Command::NextAgent), Some(1));
        assert_eq!(rows[1].1, "p2");
        let number = step(&rows, None, 0, Command::AgentNumber(2));
        assert_eq!(number.map(|index| rows[index].1), Some("p2"));
        // A collapsed group is still stepped into.
        view.collapsed_agent_groups.insert((
            view.endpoints[0].id.clone(),
            view.live.snapshot.as_ref().unwrap().boot_id.clone(),
            "w1".into(),
        ));
        assert_eq!(panes(view), ["p0", "p2", "p1", "p3"]);
        let snapshot = view.live.snapshot.as_mut().unwrap();
        let snapshot = Arc::make_mut(snapshot);
        snapshot.agent_view_label = Some("flat".into());
        snapshot.agent_order = ["p0", "p1", "p2", "p3"].map(str::to_owned).into();
        assert_eq!(panes(view), ["p0", "p1", "p2", "p3"]);
        // The plugin still owns the order while the panel is hidden.
        view.config.show_agents = false;
        assert_eq!(panes(view), ["p0", "p1", "p2", "p3"]);
    });
}
