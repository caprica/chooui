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

//! Domain models and core data structures.
//!
//! This module defines the central entities of the application—such as Art
//! Artists, Albums, and Tracks—representing the underlying data schema used
//! for metadata management and playback.

#[derive(Debug, Clone)]
pub struct Artist {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Album {
    pub id: i32,
    pub title: String,
    pub artist_id: i32,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub id: i32,
    pub title: String,
    pub track_number: i32,
    pub album_id: i32,
    pub filename: String,
}

#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub track_id: i32,
    pub track_title: String,
    pub track_number: i32,
    pub duration: i64,
    pub genre: Option<String>,
    pub year: Option<i64>,
    pub album_title: String,
    pub artist_name: String,
    pub filename: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct SearchQuery {
    pub(crate) search: String,
    pub(crate) artist: String,
    pub(crate) album: String,
    pub(crate) track: String
}

impl SearchQuery {
    pub(crate) fn for_artist(artist: String) -> Self {
        Self {
            search: String::default(),
            artist: artist,
            album: String::default(),
            track: String::default(),
        }
    }

    pub(crate) fn for_album(album: String) -> Self {
        Self {
            search: String::default(),
            artist: String::default(),
            album: album,
            track: String::default(),
        }
    }

    pub(crate) fn for_track(track: String) -> Self {
        Self {
            search: String::default(),
            artist: String::default(),
            album: String::default(),
            track: track,
        }
    }
}
