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

//! Media browser queue management.
//!
//! This module provides state for the media player queue, managing a list of
//! tracks queued for playback.

use std::{collections::{HashSet, VecDeque}, sync::{Arc, Mutex}};

use rand::{rng, seq::SliceRandom};

use crate::model::TrackInfo;

pub(crate) struct Queue {
    tracks: Arc<Mutex<Vec<TrackInfo>>>,
    queued: VecDeque<TrackInfo>,
    played: Vec<TrackInfo>,
}

impl Queue {
    pub(crate) fn new() -> Self {
        Self {
            tracks: Arc::new(Mutex::new(vec![])),
            queued: VecDeque::new(),
            played: Vec::new(),
        }
    }

    pub(crate) fn add_tracks(&mut self, tracks: Vec<TrackInfo>) {
        self.queued.extend(tracks);

        self.sync_tracks();
    }

    pub(crate) fn remove_tracks(&mut self, track_ids: Vec<i32>) {
        let ids_to_remove: HashSet<i32> = track_ids.into_iter().collect();
        self.played.retain(|track| !ids_to_remove.contains(&track.track_id));
        self.queued.retain(|track| !ids_to_remove.contains(&track.track_id));

        self.sync_tracks();
    }

    pub(crate) fn shuffle(&mut self) {
        let mut rng = rng();
        let slice = self.queued.make_contiguous();
        slice.shuffle(&mut rng);

        self.sync_tracks();
    }

    pub(crate) fn clear(&mut self) {
        self.queued.clear();
        self.played.clear();

        self.sync_tracks();
    }

    pub(crate) fn reset(&mut self) {
        while let Some(track) = self.played.pop() {
            self.queued.push_front(track);
        }
    }

    pub(crate) fn current(&self) -> Option<&TrackInfo> {
        self.played.last()
    }

    pub(crate) fn next(&mut self) -> Option<&TrackInfo> {
        if let Some(track) = self.queued.pop_front() {
            self.played.push(track);
        }

        self.played.last()
    }

    pub(crate) fn previous(&mut self) -> Option<&TrackInfo> {
        if let Some(track) = self.played.pop() {
            self.queued.push_front(track);
        }

        self.played.last()
    }

    pub(crate) fn tracks(&self) -> Arc<Mutex<Vec<TrackInfo>>> {
        Arc::clone(&self.tracks)
    }

    fn sync_tracks(&self) {
        let mut locked_tracks = self.tracks.lock().unwrap();
        locked_tracks.clear();
        locked_tracks.extend(self.played.iter().cloned());
        locked_tracks.extend(self.queued.iter().cloned());
    }
}
