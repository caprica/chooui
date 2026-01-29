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

//! Playlist management and track sequence state.
//!
//! This module coordinates the `TrackTableState` for the active playlist,
//! providing methods to populate, append, and manage the collection of tracks
//! queued for playback.

mod event;
mod render;

use crate::model::TrackInfo;

use crate::components::TrackTableState;

pub(crate) struct PlaylistView {
    pub table_state: TrackTableState,
    pub is_active: bool,
}

impl PlaylistView {
    pub(crate) fn new() -> Self {
        Self {
            table_state: TrackTableState::new(),
            is_active: false
        }
    }

    pub(crate) fn set_tracks(&mut self, tracks: Vec<TrackInfo>) {
        self.table_state.tracks = tracks;
    }

    pub(crate) fn add_tracks(&mut self, tracks: Vec<TrackInfo>) {
        self.table_state.tracks.extend(tracks);
    }
}
