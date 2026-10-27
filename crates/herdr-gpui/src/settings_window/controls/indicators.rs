//! Agent status indicators: which of this client's sets draws status, and
//! the edge rail. The Indicators page renders the choice; this keeps how a
//! pick is saved.
use super::*;
use crate::config::{StatusCounts, StatusIcons, preferences::Preference};

/// The name the Indicators page gives a set.
pub(in crate::settings_window) fn set_label(icons: StatusIcons) -> &'static str {
    match icons {
        StatusIcons::Dots => "Dots",
        StatusIcons::Solid => "Solid",
        StatusIcons::Outline => "Outline",
        StatusIcons::Tiles => "Tiles",
        StatusIcons::Wave => "Wave",
        StatusIcons::Braille => "Braille",
        StatusIcons::BrailleDots => "Braille Dots",
        StatusIcons::Orca => "Orca",
    }
}

/// One line on what sets a set apart, under its preview.
pub(in crate::settings_window) fn set_description(icons: StatusIcons) -> &'static str {
    match icons {
        StatusIcons::Dots => "Small dots; a ring with a turning arc while working.",
        StatusIcons::Solid => "Filled discs with a cut-out mark; a crescent when idle.",
        StatusIcons::Outline => "The same marks in outline, with an open arc spinning.",
        StatusIcons::Tiles => "Each mark on a tile tinted with its status color.",
        StatusIcons::Wave => "Superset's wave grid; every status drawn in its cells.",
        StatusIcons::Braille => "Braille cells; blocked fills and empties dot by dot.",
        StatusIcons::BrailleDots => "Dots with a braille spinner; a crossed disc when blocked.",
        StatusIcons::Orca => "Orca's stepping ring, question bubble, and ringed check.",
    }
}

/// The name the Indicators page gives a counts style.
pub(in crate::settings_window) fn count_label(style: StatusCounts) -> &'static str {
    match style {
        StatusCounts::Dots => "Dots",
        StatusCounts::Chips => "Chips",
        StatusCounts::Split => "Split",
        StatusCounts::Spotlight => "Spotlight",
    }
}

/// One line on what sets a counts style apart, under its preview.
pub(in crate::settings_window) fn count_description(style: StatusCounts) -> &'static str {
    match style {
        StatusCounts::Dots => "Colored dots and numbers; idle and unknown stay muted.",
        StatusCounts::Chips => "Live statuses in tinted pills; idle and unknown stay plain.",
        StatusCounts::Split => "Live counts on the left, quiet counts pushed to the right.",
        StatusCounts::Spotlight => "One pill for the most urgent status, the rest as small counts.",
    }
}

impl SettingsWindow {
    /// Picks the set status is drawn with. The set already in use saves
    /// nothing. A retired `rail` value turned the rail on with no `edge_rail`
    /// key, so an on rail is written with the set to stay on once that value
    /// is replaced.
    pub(in crate::settings_window) fn choose_status_icons(
        &mut self,
        icons: StatusIcons,
        cx: &mut Context<Self>,
    ) {
        if self.config.status_icons == icons {
            return;
        }
        let edits: Vec<Preference> = self
            .config
            .edge_rail
            .then_some(Preference::EdgeRail(true))
            .into_iter()
            .chain([Preference::StatusIcons(icons)])
            .collect();
        #[cfg(test)]
        if let Some(io) = self.controls.preference_io.clone() {
            let load = io.load.clone();
            self.save_with(
                move || edits.into_iter().try_for_each(|edit| (io.write)(edit)),
                move || load(),
                false,
                cx,
            );
            return;
        }
        self.save_native(
            move || edits.into_iter().try_for_each(Config::save_preference),
            cx,
        );
    }

    /// Picks how a group's agent counts are drawn. The style already in use
    /// saves nothing.
    pub(in crate::settings_window) fn choose_status_counts(
        &mut self,
        style: StatusCounts,
        cx: &mut Context<Self>,
    ) {
        if self.config.status_counts == style {
            return;
        }
        let edit = Preference::StatusCounts(style);
        #[cfg(test)]
        if let Some(io) = self.controls.preference_io.clone() {
            let load = io.load.clone();
            self.save_with(move || (io.write)(edit), move || load(), false, cx);
            return;
        }
        self.save_native(move || Config::save_preference(edit), cx);
    }
}
