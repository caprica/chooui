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

//! Media browser state management.
//!
//! This module provides state for the media browser widget, managing selected
//! artist, album and tracks, and navigating between the various panes in the
//! UI.

use ratatui::widgets::ListState;

use crate::model::{Album, Artist, Track};

#[derive(Default, Eq, PartialEq)]
pub(crate) enum MediaBrowserPane {
    #[default]
    Artist,
    Album,
    Track
}

#[derive(Default)]
pub(crate) struct MediaBrowser {
    pub(crate) active_pane: MediaBrowserPane,

    pub(crate) artists: Vec<Artist>,
    pub(crate) albums: Vec<Album>,
    pub(crate) tracks: Vec<Track>,

    pub(crate) artists_state: ListState,
    pub(crate) albums_state: ListState,
    pub(crate) tracks_state: ListState,
}

impl MediaBrowser {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn set_pane(&mut self, pane: MediaBrowserPane) {
        self.active_pane = pane;
    }

    pub(crate) fn next_pane(&mut self) {
        self.active_pane = match self.active_pane {
            MediaBrowserPane::Artist => MediaBrowserPane::Album,
            MediaBrowserPane::Album => MediaBrowserPane::Track,
            MediaBrowserPane::Track => MediaBrowserPane::Artist,
        };
    }

    pub(crate) fn previous_pane(&mut self) {
        self.active_pane = match self.active_pane {
            MediaBrowserPane::Artist => MediaBrowserPane::Track,
            MediaBrowserPane::Album => MediaBrowserPane::Artist,
            MediaBrowserPane::Track => MediaBrowserPane::Album,
        };
    }

    pub(crate) fn selected_artist_id(&self) -> Option<i32> {
        let index = self.artists_state.selected()?;
        self.artists.get(index).map(|artist| artist.id)
    }

    pub(crate) fn selected_album_id(&self) -> Option<i32> {
        let index = self.albums_state.selected()?;
        self.albums.get(index).map(|album| album.id)
    }

    pub(crate) fn selected_track_id(&self) -> Option<i32> {
        let index = self.tracks_state.selected()?;
        self.tracks.get(index).map(|track| track.id)
    }

    pub(crate) fn next_artist(&mut self) { Self::next(&mut self.artists_state, self.artists.len()); }
    pub(crate) fn previous_artist(&mut self) { Self::previous(&mut self.artists_state, self.artists.len()); }

    pub(crate) fn next_album(&mut self) { Self::next(&mut self.albums_state, self.albums.len()); }
    pub(crate) fn previous_album(&mut self) { Self::previous(&mut self.albums_state, self.albums.len()); }

    pub(crate) fn next_track(&mut self) { Self::next(&mut self.tracks_state, self.tracks.len()); }
    pub(crate) fn previous_track(&mut self) { Self::previous(&mut self.tracks_state, self.tracks.len()); }

    pub(crate) fn set_artists(&mut self, artists: Vec<Artist>) {
        self.artists = artists;
        self.artists_state.select((!self.artists.is_empty()).then_some(0));
    }

    pub(crate) fn set_albums(&mut self, albums: Vec<Album>) {
        self.albums = albums;
        self.albums_state.select((!self.albums.is_empty()).then_some(0));
    }

    pub(crate) fn set_tracks(&mut self, tracks: Vec<Track>) {
        self.tracks = tracks;
        self.tracks_state.select((!self.tracks.is_empty()).then_some(0));
    }

    fn next(state: &mut ListState, len: usize) {
        if len == 0 { return; }
        let i = match state.selected() {
            Some(i) => if i >= len - 1 { 0 } else { i + 1 },
            None => 0,
        };
        state.select(Some(i));
    }

    fn previous(state: &mut ListState, len: usize) {
        if len == 0 { return; }
        let i = match state.selected() {
            Some(i) => if i == 0 { len - 1 } else { i - 1 },
            None => 0,
        };
        state.select(Some(i));
    }
}
