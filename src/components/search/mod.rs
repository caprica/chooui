// Copyright (C) 2026  Caprica Software Limited
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Search results view and track selection management.
//!
//! This module coordinates the `TrackTableState` for search results,
//! providing methods to update track listings, manage selection lifecycle,
//! and retrieve selected track data.

mod event;
mod render;

use crate::{components::TrackTableState, model::TrackInfo};

pub(crate) struct SearchView {
    pub table_state: TrackTableState,
    pub is_active: bool,
}

impl SearchView {
    pub(crate) fn new() -> Self {
        Self {
            table_state: TrackTableState::new(),
            is_active: false
        }
    }

    pub(crate) fn set_tracks(&mut self, tracks: Vec<TrackInfo>) {
        // Set the new tracks and reset any existing selection.
        self.table_state.tracks = tracks;
        self.table_state.selection.clear();

        // Set a default selection if possible
        if self.table_state.tracks.is_empty() {
             self.table_state.table_state.select(None);
        } else {
            self.table_state.table_state.select(Some(0));
        }
    }

    pub(crate) fn clone_selected_tracks(&self) -> Vec<TrackInfo> {
        self.table_state.tracks.iter()
            .filter(|t| self.table_state.selection.contains(&t.track_id))
            .cloned()
            .collect()
    }

    pub(crate) fn clear_selection(&mut self) {
        self.table_state.selection.clear();
    }
}
