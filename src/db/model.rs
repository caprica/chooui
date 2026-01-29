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

//! Database row mapping for domain models.
//!
//! This module provides the conversion logic between raw SQLite result rows
//! and high-level domain models, ensuring type-safe extraction of model
//! attributes from database queries.

use rusqlite::Row;

use crate::model::TrackInfo;

impl TrackInfo {

    /// Maps an SQLite row to a [`TrackInfo`] instance.
    ///
    /// This is a helper function designed to be used with [`rusqlite::Statement::query_map`].
    ///
    /// # Errors
    ///
    /// Returns a [`rusqlite::Error`] if:
    /// * The row does not contain enough columns.
    /// * The data in a column cannot be converted to the required Rust type.
    pub(crate) fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            artist_name: row.get(0)?,
            album_title: row.get(1)?,
            track_id: row.get(2)?,
            track_number: row.get(3)?,
            track_title: row.get(4)?,
            duration: row.get(5)?,
            year: row.get(6)?,
            genre: row.get(7)?,
            filename: row.get(8)?,
        })
    }
}
