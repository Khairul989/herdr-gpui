//! `status_counts`: how a workspace agent group's header draws its per-status
//! counts.

use serde::Deserialize;

/// Declares each style once with its config spelling, so parsing, saving, and
/// the settings list cannot drift.
macro_rules! status_counts {
    (
        $(#[doc = $first_doc:literal])* $first:ident => $first_name:literal,
        $($(#[doc = $doc:literal])* $style:ident => $name:literal,)+
    ) => {
        #[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
        pub enum StatusCounts {
            $(#[doc = $first_doc])* #[default] #[serde(rename = $first_name)] $first,
            $($(#[doc = $doc])* #[serde(rename = $name)] $style,)+
        }

        impl StatusCounts {
            /// Every style, in the order the settings window shows them.
            pub const ALL: &[Self] = &[Self::$first, $(Self::$style,)+];

            /// The config spelling, which deserializing reads back.
            pub fn as_str(self) -> &'static str {
                match self {
                    Self::$first => $first_name,
                    $(Self::$style => $name,)+
                }
            }
        }
    };
}

// Shown to users in this order, so not sorted. The first is the default.
status_counts! {
    /// The rows' own dots with colored numbers; idle and unknown stay muted.
    Dots => "dots",
    /// Live statuses in tinted pills; idle and unknown stay plain.
    Chips => "chips",
    /// Live counts on the left, quiet counts pushed to the right.
    Split => "split",
    /// One pill for the most urgent status, the rest as small counts.
    Spotlight => "spotlight",
}
