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

use rusqlite::{
    Result, Row, ToSql,
    types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef},
};

use crate::model::{Rating, TrackInfo};

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
            durable_id: row.get(3)?,
            track_number: row.get(4)?,
            track_title: row.get(5)?,
            duration: row.get(6)?,
            year: row.get(7)?,
            genre: row.get(8)?,
            filename: row.get(9)?,
            play_count: row.get(10)?,
            rating: row.get(11)?,
        })
    }
}

impl ToSql for Rating {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        let val = match self {
            Rating::Like => 1,
            Rating::Neutral => 0,
            Rating::Dislike => -1,
        };
        Ok(ToSqlOutput::from(val))
    }
}

impl FromSql for Rating {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_i64()? {
            1 => Ok(Rating::Like),
            0 => Ok(Rating::Neutral),
            -1 => Ok(Rating::Dislike),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}
