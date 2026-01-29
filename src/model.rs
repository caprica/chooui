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
