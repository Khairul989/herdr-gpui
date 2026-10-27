//! How a workspace group's agent counts are drawn: the rows' own dots with
//! numbers, tinted chips, a split with quiet counts pushed right, or one
//! spotlight pill naming the most urgent status.

use super::{
    agents::{AgentGroupCounts, Indicators, status_text},
    label_text, line_height,
};
use crate::config::{FontConfig, StatusCounts, Theme, corners};
use gpui::{prelude::*, *};
use herdr_client::protocol::AgentStatus;

/// The buckets in the order a group shows them, what needs attention first.
const URGENT: [AgentStatus; 5] = [
    AgentStatus::Blocked,
    AgentStatus::Working,
    AgentStatus::Done,
    AgentStatus::Idle,
    AgentStatus::Unknown,
];

pub(crate) fn counts_line(
    counts: &AgentGroupCounts,
    style: StatusCounts,
    font: &FontConfig,
    indicators: Indicators,
    theme: &Theme,
) -> Div {
    let buckets = shown_counts(counts);
    match style {
        StatusCounts::Dots => line(font, style).children(
            buckets
                .into_iter()
                .map(|(status, count)| pair(status, count, indicators, theme)),
        ),
        StatusCounts::Chips => {
            line(font, style).children(buckets.into_iter().map(|(status, count)| {
                if live(status) {
                    chip(status, count, indicators)
                } else {
                    pair(status, count, indicators, theme)
                }
            }))
        }
        // The spacer keeps quiet counts on the right even when nothing is
        // live, so every group's noise sits in the same place.
        StatusCounts::Split => line(font, style)
            .child(
                group().children(
                    buckets
                        .iter()
                        .filter(|(status, _)| live(*status))
                        .map(|&(status, count)| pair(status, count, indicators, theme)),
                ),
            )
            .child(div().flex_1())
            .child(
                group().children(
                    buckets
                        .iter()
                        .filter(|(status, _)| !live(*status))
                        .map(|&(status, count)| pair(status, count, indicators, theme)),
                ),
            ),
        StatusCounts::Spotlight => {
            let Some((status, count)) = focus(counts) else {
                return line(font, style);
            };
            let lead = if live(status) {
                pill(status, count, lead_word(status), indicators)
            } else {
                quiet_pill(status, count, theme)
            };
            line(font, style).child(lead).child(div().flex_1()).child(
                group().children(
                    buckets
                        .into_iter()
                        .filter(move |(remaining, _)| *remaining != status)
                        .map(move |(remaining, count)| pair(remaining, count, indicators, theme)),
                ),
            )
        }
    }
}

/// The buckets a group shows, urgency first, leaving out empty ones.
pub(crate) fn shown_counts(counts: &AgentGroupCounts) -> Vec<(AgentStatus, usize)> {
    URGENT
        .into_iter()
        .map(|status| (status, count_of(counts, status)))
        .filter(|(_, count)| *count > 0)
        .collect()
}

/// The status a spotlight pill names: the most urgent live bucket, else the
/// first quiet one that is non-empty.
pub(crate) fn focus(counts: &AgentGroupCounts) -> Option<(AgentStatus, usize)> {
    let buckets = shown_counts(counts);
    buckets
        .iter()
        .find(|(status, _)| live(*status))
        .or_else(|| buckets.iter().find(|(status, _)| !live(*status)))
        .copied()
}

/// What a live status means for the person reading the pill; the rest are
/// the daemon's own names.
fn lead_word(status: AgentStatus) -> &'static str {
    match status {
        AgentStatus::Blocked => "needs attention",
        _ => status_text(status),
    }
}

/// The line every style shares: one wrapping row of marks at a group's size.
fn line(font: &FontConfig, style: StatusCounts) -> Div {
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap_x(px(10.))
        .text_size(px(font.size * 0.8))
        .line_height(px(line_height(font) * 0.85))
        .debug_selector(|| format!("agent-group-counts-{}", style.as_str()))
}

/// A wrapping run of marks inside a line.
fn group() -> Div {
    div().flex().flex_wrap().items_center().gap_x(px(10.))
}

fn live(status: AgentStatus) -> bool {
    matches!(
        status,
        AgentStatus::Blocked | AgentStatus::Working | AgentStatus::Done
    )
}

fn count_of(counts: &AgentGroupCounts, status: AgentStatus) -> usize {
    match status {
        AgentStatus::Blocked => counts.blocked,
        AgentStatus::Working => counts.working,
        AgentStatus::Done => counts.done,
        AgentStatus::Idle => counts.idle,
        AgentStatus::Unknown => counts.unknown,
    }
}

/// One bucket as a dot and its number: colored for a live status, muted for
/// idle and unknown, whose marks keep the rows' own shapes.
fn pair(status: AgentStatus, count: usize, indicators: Indicators, theme: &Theme) -> Div {
    match status {
        AgentStatus::Idle => {
            let muted = theme.muted;
            div()
                .flex()
                .flex_none()
                .items_center()
                .gap(px(4.))
                .child(dot(muted, false, 8.))
                .child(
                    div()
                        .text_color(rgb(muted))
                        .child(label_text(&count.to_string())),
                )
        }
        AgentStatus::Unknown => {
            let muted = theme.muted;
            div()
                .flex()
                .flex_none()
                .items_center()
                .gap(px(4.))
                .child(dot(muted, true, 3.))
                .child(
                    div()
                        .text_color(rgb(muted))
                        .child(label_text(&count.to_string())),
                )
        }
        _ => {
            let color = indicators.color(status);
            div()
                .flex()
                .flex_none()
                .items_center()
                .gap(px(4.))
                .child(dot(color, true, 8.))
                .child(
                    div()
                        .text_color(rgb(color))
                        .child(label_text(&count.to_string())),
                )
        }
    }
}

fn dot(color: u32, filled: bool, side: f32) -> Div {
    div()
        .flex_none()
        .size(px(side))
        .rounded_full()
        .when(filled, |dot| dot.bg(rgb(color)))
        .when(!filled, |dot| dot.border_1().border_color(rgb(color)))
}

/// A live bucket in a tinted pill sized for a small badge.
fn chip(status: AgentStatus, count: usize, indicators: Indicators) -> Div {
    let color = indicators.color(status);
    div()
        .flex()
        .flex_none()
        .items_center()
        .gap(px(4.))
        .px(px(5.))
        .py(px(1.))
        .rounded(px(corners::SMALL))
        .bg(rgba((color << 8) | 0x26))
        .border_1()
        .border_color(rgba((color << 8) | 0x78))
        .child(dot(color, true, 6.))
        .child(
            div()
                .text_color(rgb(color))
                .child(label_text(&count.to_string())),
        )
}

/// The spotlight's lead: its count in bold, then the word for the status.
fn pill(status: AgentStatus, count: usize, word: &'static str, indicators: Indicators) -> Div {
    let color = indicators.color(status);
    div()
        .flex()
        .flex_none()
        .items_center()
        .gap(px(5.))
        .px(px(6.))
        .py(px(1.))
        .rounded(px(corners::CONTROL))
        .bg(rgba((color << 8) | 0x26))
        .border_1()
        .border_color(rgba((color << 8) | 0x78))
        .child(dot(color, true, 6.))
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(color))
                .child(label_text(&count.to_string())),
        )
        .child(div().text_color(rgb(color)).child(label_text(word)))
}

/// The spotlight's lead when nothing is live: the quiet count on the row's
/// own surface, so it reads as background rather than attention.
fn quiet_pill(status: AgentStatus, count: usize, theme: &Theme) -> Div {
    let muted = theme.muted;
    div()
        .flex()
        .flex_none()
        .items_center()
        .gap(px(5.))
        .px(px(6.))
        .py(px(1.))
        .rounded(px(corners::CONTROL))
        .bg(rgb(theme.active))
        .child(dot(muted, false, 6.))
        .child(
            div()
                .text_color(rgb(muted))
                .child(label_text(&format!("{count} {}", status_text(status)))),
        )
}
