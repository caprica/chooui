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

use std::collections::{HashSet, VecDeque};
use rand::{rng, seq::SliceRandom};

use crate::model::TrackInfo;

pub(crate) struct Queue {
    queued: VecDeque<TrackInfo>,
    played: Vec<TrackInfo>,
    pub(crate) current: Option<TrackInfo>
}

impl Queue {
    pub(crate) fn new() -> Self {
        Self {
            queued: VecDeque::new(),
            played: Vec::new(),
            current: None
        }
    }

    pub(crate) fn add_tracks(&mut self, tracks: Vec<TrackInfo>) {
        self.queued.extend(tracks);
    }

    pub(crate) fn remove_tracks(&mut self, track_ids: Vec<i32>) {
        let ids_to_remove: HashSet<i32> = track_ids.into_iter().collect();

        if let Some(ref track) = self.current {
            if ids_to_remove.contains(&track.track_id) {
                self.current = None;
            }
        }

        self.played.retain(|track| !ids_to_remove.contains(&track.track_id));
        self.queued.retain(|track| !ids_to_remove.contains(&track.track_id));
    }

    pub(crate) fn shuffle(&mut self) {
        let mut rng = rng();
        let mut tmp: Vec<TrackInfo> = self.queued.drain(..).collect();
        tmp.shuffle(&mut rng);
        self.queued.extend(tmp)
    }

    pub(crate) fn clear(&mut self) {
        self.queued.clear();
    }

    pub(crate) fn reset(&mut self) {
        if let Some(track) = self.current.take() {
            self.queued.push_front(track);
        }

        while let Some(track) = self.played.pop() {
            self.queued.push_front(track);
        }
    }

    pub(crate) fn current(&self) -> Option<&TrackInfo> {
        self.current.as_ref()
    }

    pub(crate) fn played(&self) -> Vec<TrackInfo> {
        self.played.iter().cloned().collect()
    }

    pub(crate) fn queued(&self) -> Vec<TrackInfo> {
        self.queued.iter().cloned().collect()
    }

    pub(crate) fn next(&mut self) -> Option<&TrackInfo> {
        if let Some(track) = self.current.take() {
            self.played.push(track);
        }

        self.current = self.queued.pop_front();

        self.current.as_ref()
    }

    pub(crate) fn previous(&mut self) -> Option<&TrackInfo> {
        if self.played.is_empty() {
            return self.current.as_ref();
        }

        if let Some(track) = self.current.take() {
            self.queued.push_front(track);
        }

        self.current = self.played.pop();

        self.current.as_ref()
    }
}
