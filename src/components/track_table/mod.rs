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

mod event;
mod render;

use std::{collections::HashSet, sync::{Arc, Mutex}};

use ratatui::widgets::TableState;

use crate::model::TrackInfo;

pub(crate) enum TrackTableAction {
    ActivateCurrent(i32),
    CommitSelection(HashSet<i32>),
}

pub(crate) struct TrackTable {
    tracks: Arc<Mutex<Vec<TrackInfo>>>,
    selection: HashSet<i32>,
    table_state: TableState,
}

impl TrackTable {
    pub(crate) fn new(tracks: Arc<Mutex<Vec<TrackInfo>>>) -> Self {
        Self {
            tracks,
            selection: HashSet::new(),
            table_state: TableState::new(),
        }
    }

    fn goto_next(&mut self) {
        let tracks = self.tracks.lock().unwrap();
        let len = tracks.len();
        if len == 0 { return; }
        let i = match self.table_state.selected() {
            Some(i) => if i >= len - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn goto_previous(&mut self) {
        let tracks = self.tracks.lock().unwrap();
        let len = tracks.len();
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
        let tracks = self.tracks.lock().unwrap();
        if let Some(selected_index) = self.table_state.selected() {
            if let Some(track) = tracks.get(selected_index) {
                let track_id = track.track_id;
                if !self.selection.insert(track_id) {
                    self.selection.remove(&track_id);
                }
            }
        }
    }

    fn select_all(&mut self) {
        let tracks = self.tracks.lock().unwrap();
        self.selection.extend(tracks.iter().map(|t| t.track_id));
    }

    fn select_inverse(&mut self) {
        let tracks = self.tracks.lock().unwrap();
        for track in tracks.iter() {
            let track_id = track.track_id;
            if !self.selection.insert(track_id) {
                self.selection.remove(&track_id);
            }
        }
    }

    fn select_none(&mut self) {
        self.selection.clear();
    }

    pub(crate) fn reset_table_selection(&mut self) {
        self.goto_first();
    }

    pub(crate) fn ensure_table_selection(&mut self) {
        if self.table_state.selected().is_none() {
            self.goto_first();
        }
    }

    pub(crate) fn clear_selection(&mut self) {
        self.selection.clear();
    }

    pub(crate) fn clone_selected_tracks(&self) -> Vec<TrackInfo> {
        let tracks = self.tracks.lock().unwrap();
        tracks.iter()
            .filter(|t| self.selection.contains(&t.track_id))
            .cloned()
            .collect()
    }

    pub(crate) fn clone_track(&self, track_id: i32) -> Option<TrackInfo> {
        let locked = self.tracks.lock().unwrap();
        locked.iter()
            .find(|t| t.track_id == track_id)
            .cloned()
    }

    pub(crate) fn clone_tracks(&self, track_ids: HashSet<i32>) -> Vec<TrackInfo> {
        let locked = self.tracks.lock().unwrap();
        locked.iter()
            .filter(|t| track_ids.contains(&t.track_id))
            .cloned()
            .collect()
    }
}
