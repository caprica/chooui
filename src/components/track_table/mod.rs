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

//! Interactive track table widget and state management.
//!
//! This module provides a reusable table component for displaying and
//! selecting tracks. It separates persistent state (`TrackTableState`) from
//! the transient widget view (`TrackTable`) and uses a delegate pattern to
//! decouple event handling from the main application logic.

mod event;
mod render;

use std::collections::HashSet;

use ratatui::widgets::TableState;

use crate::model::TrackInfo;

pub(crate) trait TrackTableDelegate {
    fn on_activate_selection(&self);
}

pub(crate) struct TrackTableState {
    pub(crate) tracks: Vec<TrackInfo>,
    pub(crate) selection: HashSet<i32>,
    pub(crate) table_state: TableState,
}

impl TrackTableState {
    pub(crate) fn new() -> Self {
        Self {
            tracks: vec![],
            selection: HashSet::new(),
            table_state: TableState::new(),
        }
    }

    pub(crate) fn as_widget(&mut self) -> TrackTable<'_> {
        TrackTable {
            tracks: &self.tracks,
            selection: &mut self.selection,
            table_state: &mut self.table_state
        }
    }
}

pub(crate) struct TrackTable<'a> {
    tracks: &'a [TrackInfo],
    selection: &'a mut HashSet<i32>,
    table_state: &'a mut TableState,
}

impl<'a> TrackTable<'a> {
    fn goto_next(&mut self) {
        let len = self.tracks.len();
        if len == 0 { return; }
        let i = match self.table_state.selected() {
            Some(i) => if i >= len - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn goto_previous(&mut self) {
        let len = self.tracks.len();
        if len == 0 { return; }
        let i = match self.table_state.selected() {
            Some(i) => if i == 0 { len - 1 } else { i - 1 },
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn goto_first(&mut self) {
        self.table_state.select_first();
    }

    fn goto_last(&mut self) {
        self.table_state.select_last();
    }

    fn toggle_select_current(&mut self) {
        if let Some(selected_index) = self.table_state.selected() {
            if let Some(track) = self.tracks.get(selected_index) {
                let track_id = track.track_id;
                if !self.selection.insert(track_id) {
                    self.selection.remove(&track_id);
                }
            }
        }
    }

    fn select_all(&mut self) {
        self.selection.extend(self.tracks.iter().map(|t| t.track_id));
    }

    fn select_inverse(&mut self) {
        for track in self.tracks.iter() {
            let track_id = track.track_id;
            if !self.selection.insert(track_id) {
                self.selection.remove(&track_id);
            }
        }
    }

    fn select_none(&mut self) {
        self.selection.clear();
    }
}
