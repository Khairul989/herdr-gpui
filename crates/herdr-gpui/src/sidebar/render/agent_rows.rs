//! The separate Agents panel: singleton rows and conditional workspace groups,
//! retaining configured membership and the panel's ordering source.

use crate::{
    HerdrWindow, NavigationTarget,
    sidebar::{
        Indicators, agent_name,
        agents::{
            AgentGroupCounts, agent_group_key, agent_place, pane_name, state_label, status_text,
            workspace_agent_groups,
        },
        cell::{AgentRow, Cell, Fold, RowContext, RowData, RowState, layout_for},
        layout::SidebarLook,
        line_height, tokens, wash,
    },
};
use gpui::{prelude::*, *};
use herdr_client::protocol::{ClientShellAgent, ClientShellSnapshot};

/// Alpha of the accent band behind a workspace group's dropdown header: a
/// tint that sets the header apart from the rows it heads, never a fill.
const HEADER_TINT: u32 = 0x1c;
/// The band under the pointer, one step stronger.
const HEADER_HOVER_TINT: u32 = 0x2e;

/// How much of where an agent runs its row names, given what the rows above
/// it already say.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PlaceShown {
    /// Its workspace and tab: nothing above names them.
    Full,
    /// Its tab alone, under a group header that names the workspace and host.
    Tab,
    /// Nothing, under the workspace's own row.
    Hidden,
}

impl HerdrWindow {
    /// Groups only listed agents, after configured-row filtering. The returned
    /// count and focus index describe actual scroll children, including headers.
    pub(super) fn append_agent_rows(
        &self,
        mut list: Stateful<Div>,
        indicators: Indicators,
        look: SidebarLook,
        width: f32,
        flat: bool,
        cx: &mut Context<Self>,
    ) -> (Stateful<Div>, usize, Option<usize>) {
        let font = &self.config.sidebar;
        let theme = &self.theme;
        let multi = self.endpoints.len() > 1;
        let mut count = 0;
        let mut focused = None;
        let (listed, mut lines): (Vec<_>, Vec<_>) = self
            .panel_agents()
            .into_iter()
            .filter_map(|(index, agent)| {
                let endpoint = self.endpoints.get(index)?;
                let live = if index == self.selected_endpoint {
                    &self.live
                } else {
                    &endpoint.live
                };
                let snapshot = live.snapshot.as_deref()?;
                let host = (multi && endpoint.id != crate::endpoint::LOCAL)
                    .then_some(endpoint.label.as_str());
                let lines = self.configured_agent_lines(agent, snapshot, host)?;
                Some(((index, agent, snapshot, host), lines))
            })
            .unzip();
        let keys: Vec<_> = listed
            .iter()
            .map(|(index, agent, snapshot, _)| {
                agent_group_key(&self.endpoints[*index].id, snapshot, agent)
            })
            .collect();
        for group in workspace_agent_groups(&keys, self.agent_sort, flat) {
            let first = group[0];
            let (index, agent, snapshot, host) = &listed[first];
            let endpoint = &self.endpoints[*index];
            let selected = *index == self.selected_endpoint;
            let mut row_cx = RowContext {
                indicators,
                font,
                theme,
                look,
                width,
                nest: 0.,
                mark: wash::HostMark::resolve(
                    &self.config.sidebar_style,
                    &endpoint.label,
                    selected,
                    theme,
                ),
                host: *host,
            };
            let grouped = group.len() >= 2;
            let collapsed = grouped && self.collapsed_agent_groups.contains(&keys[first]);
            if grouped {
                let selected = group.iter().any(|&position| {
                    let (index, agent, ..) = &listed[position];
                    *index == self.selected_endpoint && agent.focused
                });
                if selected && collapsed {
                    focused = Some(count);
                }
                let workspace =
                    agent_place(agent, snapshot).map_or("Workspace", |(label, _)| label);
                let label = host.map_or_else(
                    || workspace.to_owned(),
                    |host| format!("{workspace} · {host}"),
                );
                let counts = AgentGroupCounts::from_statuses(
                    group
                        .iter()
                        .map(|&position| listed[position].1.agent_status),
                );
                list = list.child(self.agent_group_header(
                    &keys[first],
                    label,
                    counts,
                    selected && collapsed,
                    &row_cx,
                    cx,
                ));
                count += 1;
                if collapsed {
                    continue;
                }
                row_cx.nest = look.nest_indent();
                row_cx.host = None;
            }
            let shown = if grouped {
                PlaceShown::Tab
            } else {
                PlaceShown::Full
            };
            for position in group {
                let (index, agent, snapshot, _) = &listed[position];
                // Each group member is visited once, so its lines move out.
                let lines = std::mem::take(&mut lines[position]);
                if *index == self.selected_endpoint && agent.focused {
                    focused = Some(count);
                }
                let gap = self.agent_row_gap(count);
                count += 1;
                list = list.child(
                    self.agent_cell(*index, agent, snapshot, &row_cx, lines, gap, shown, cx),
                );
            }
        }
        (list, count, focused)
    }

    pub(in crate::sidebar) fn toggle_agent_group(
        &mut self,
        key: &(String, String, String),
        cx: &mut Context<Self>,
    ) {
        if !self.collapsed_agent_groups.remove(key) {
            self.collapsed_agent_groups.insert(key.clone());
        }
        // A disclosure changes scroll-child indices even if pane focus holds.
        self.sidebar_revealed[1].set(None);
        cx.notify();
    }

    fn agent_group_header(
        &self,
        key: &(String, String, String),
        label: String,
        counts: AgentGroupCounts,
        selected: bool,
        row_cx: &RowContext,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let collapsed = self.collapsed_agent_groups.contains(key);
        let spoken = format!("{label}, {}", counts.description());
        let hint = SharedString::from(spoken.clone());
        let (foreground, surface) = (row_cx.theme.foreground, row_cx.theme.surface);
        let primary = row_cx.theme.primary();
        let accent = row_cx.theme.ink(primary);
        let band = rgba((primary << 8) | HEADER_TINT);
        let hover_band = rgba((primary << 8) | HEADER_HOVER_TINT);
        let toggle_key = key.clone();
        let click_key = key.clone();
        let id = format!("agent-group-{key:?}");
        let probe = format!("agent-group-{}-{}-{}", key.0, key.1, key.2);
        let disclosure = format!("agent-group-disclosure-{}-{}-{}", key.0, key.1, key.2);
        let fold = Fold {
            id: SharedString::from(format!("fold-{key:?}")).into(),
            index: 0,
            collapsed,
            toggle: Box::new(
                cx.listener(move |this, _, _, cx| this.toggle_agent_group(&toggle_key, cx)),
            ),
        };
        let row = div()
            .id(SharedString::from(id.clone()))
            .debug_selector(move || probe.clone())
            .relative()
            .flex_none()
            .flex()
            .flex_col()
            .px(px(row_cx.look.content_x()))
            .py(px(row_cx.look.density.host_padding()))
            .bg(band)
            .hover(move |style| style.bg(hover_band))
            .cursor_pointer()
            .map(|row| {
                let highlight = row_cx.look.highlight(
                    &id,
                    RowState {
                        selected,
                        ..Default::default()
                    },
                    row_cx.theme,
                );
                row_cx.look.washed(row, &id, row_cx.mark, highlight)
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .child(
                        fold.element(row_cx.theme)
                            .text_color(rgb(accent))
                            .debug_selector(move || disclosure.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .child(crate::sidebar::label_text(&label)),
                    )
                    .child(
                        div()
                            .flex_none()
                            .text_color(rgb(accent))
                            .child(crate::sidebar::label_text(&counts.total().to_string())),
                    ),
            )
            .child(crate::sidebar::counts_line(
                &counts,
                self.config.status_counts,
                row_cx.font,
                row_cx.indicators,
                row_cx.theme,
            ))
            .tooltip(move |_, cx| {
                cx.new(|_| crate::usage::Hint {
                    text: hint.clone(),
                    foreground,
                    surface,
                })
                .into()
            })
            .on_click(cx.listener(move |this, _, _, cx| this.toggle_agent_group(&click_key, cx)));
        self.agent_group_item(row, spoken, selected, key.clone(), cx)
    }

    /// Whether the agents' rows follow a configured `[sidebar_layout.agents]`.
    fn custom_agent_rows(&self) -> bool {
        self.config.usage.inline
            && self.config.sidebar_layout.agents != crate::config::AgentLayout::default()
    }

    /// The lines a configured agent row shows, empty for Herdr's own row, or
    /// None when the configured rows leave this agent out.
    pub(super) fn configured_agent_lines(
        &self,
        agent: &ClientShellAgent,
        snapshot: &ClientShellSnapshot,
        host: Option<&str>,
    ) -> Option<Vec<Vec<tokens::ResolvedToken>>> {
        if !self.custom_agent_rows() {
            return Some(Vec::new());
        }
        tokens::agent_rows(&self.config.sidebar_layout.agents, agent, snapshot, host)
    }

    /// Space above the agent row painted `count`-th in its list: a
    /// configured `row_gap` between configured rows, none otherwise.
    pub(super) fn agent_row_gap(&self, count: usize) -> f32 {
        if self.custom_agent_rows() && count > 0 {
            f32::from(self.config.sidebar_layout.agents.row_gap) * line_height(&self.config.sidebar)
        } else {
            0.
        }
    }

    /// One agent's row, for the agents list, under its device's header or
    /// group header, or under its workspace, naming as much of its place as
    /// `shown` leaves to it: the agent of endpoint `index`, drawn in `row_cx`,
    /// that shows its pane when clicked.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn agent_cell(
        &self,
        index: usize,
        agent: &ClientShellAgent,
        snapshot: &ClientShellSnapshot,
        row_cx: &RowContext,
        lines: Vec<Vec<tokens::ResolvedToken>>,
        gap: f32,
        shown: PlaceShown,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let rows = layout_for(self.config.layout.mode);
        let multi = self.endpoints.len() > 1;
        let selected = index == self.selected_endpoint;
        let endpoint_id = self
            .endpoints
            .get(index)
            .map(|endpoint| endpoint.id.clone())
            .unwrap_or_default();
        let id = agent.pane_id.clone();
        let navigate_endpoint = endpoint_id.clone();
        let menu_endpoint = endpoint_id.clone();
        let menu_pane = id.clone();
        let menu_open = selected && self.pane_menu_target() == Some(id.as_str());
        let name = agent_name(agent);
        let pane = pane_name(agent, snapshot, name);
        let place = match shown {
            PlaceShown::Full => agent_place(agent, snapshot),
            // The tab takes the workspace's slot; without one the row keeps
            // to one line, as under the workspace's own row.
            PlaceShown::Tab => agent_place(agent, snapshot)
                .and_then(|(_, tab)| tab)
                .map(|tab| (tab, None)),
            PlaceShown::Hidden => None,
        };
        let state = state_label(agent, status_text(agent.agent_status));
        let spoken = crate::sidebar::accessibility::agent_name(
            name,
            pane.as_deref(),
            &state,
            place,
            row_cx.host,
        );
        Cell::new(
            rows,
            RowData::Agent(AgentRow {
                key: format!("agent-{id}"),
                name,
                icon: crate::icons::AgentIcon::from_identity(agent.agent.as_deref()),
                status: agent.agent_status,
                place,
                pane,
                status_text: self
                    .config
                    .sidebar_layout
                    .agents
                    .shows_status_text(agent.agent.as_deref())
                    .then_some(state),
                lines,
            }),
            row_cx,
        )
        .selected(selected && agent.focused)
        .highlighted(menu_open)
        .row()
        .when(gap > 0., |row| row.mt(px(gap)))
        .id(SharedString::from(format!("agent-{endpoint_id}-{id}")))
        .role(Role::ListItem)
        .aria_label(spoken)
        .aria_selected(selected && agent.focused)
        .map(|control| self.chrome_stop(crate::window::chrome_focus::Region::Sidebar, control))
        .when(multi, |row| {
            row.debug_selector(|| format!("agent-{endpoint_id}-{id}"))
        })
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                cx.stop_propagation();
                this.open_agent_menu(&menu_endpoint, &menu_pane, event.position, window, cx);
            }),
        )
        .on_click(cx.listener(move |this, _, window, cx| {
            this.navigate_endpoint(&navigate_endpoint, NavigationTarget::Pane(&id), cx);
            window.focus(&this.focus, cx);
        }))
    }
}
