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

//! Media search management.
//!
//! This module provides state for the media search view, managing a list of
//! tracks matching a search query.

use std::sync::{Arc, Mutex};

use crate::model::TrackInfo;

pub(crate) struct Search {
    tracks: Arc<Mutex<Vec<TrackInfo>>>,
}

impl Search {
    pub(crate) fn new() -> Self {
        Self {
            tracks: Arc::new(Mutex::new(vec![])),
        }
    }

    pub(crate) fn set_tracks(&mut self, tracks: Vec<TrackInfo>) {
        let mut lock = self.tracks.lock().unwrap();
        *lock = tracks;
    }

    pub(crate) fn tracks(&self) -> Arc<Mutex<Vec<TrackInfo>>> {
        Arc::clone(&self.tracks)
    }
}
