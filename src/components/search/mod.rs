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

use std::sync::{Arc, Mutex};

use crate::{components::TrackTable, model::TrackInfo};

pub(crate) struct SearchView {
    pub(crate) track_table: TrackTable,
    pub(crate) is_active: bool,
}

impl SearchView {
    pub(crate) fn new(tracks: Arc<Mutex<Vec<TrackInfo>>>) -> Self {
        Self {
            track_table: TrackTable::new(tracks),
            is_active: false
        }
    }
}
