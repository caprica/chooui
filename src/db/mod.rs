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

//! Data access layer.
//!
//! This module handles all interactions with the SQLite database, including
//! schema creation and fetching track metadata. It uses cached statements
//! to optimize frequently executed queries.
//!
//! # Tables
//!
//! * `artists` - Stores unique artist names.
//! * `albums` - Groups tracks under titles, linked to artists.
//! * `tracks` - Individual audio files with metadata and file paths.
//!
//! # Performance
//!
//! Most functions in this module use [`rusqlite::Connection::prepare_cached`]
//! to reduce SQL parsing overhead.

mod model;
pub(crate) mod scan;

use anyhow::{Context, Result};
use rusqlite::{Connection, params};

use crate::model::{Album, Artist, Rating, SearchQuery, Track, TrackInfo};

const MIN_SEARCH_LEN: usize = 3;

/// Opens a connection to the SQLite database and configures performance settings.
///
/// This function performs the following setup:
/// * **WAL Mode**: Enables Write-Ahead Logging for better concurrency.
/// * **Performance Tuning**: Sets synchronous mode to `NORMAL` and increases the cache size.
/// * **Constraints**: Enforces foreign key integrity.
/// * **Schema**: Executes [`create_schema`] to ensure all tables and indices exist.
///
/// # Arguments
///
/// * `path` - The file system path to the SQLite database file.
///
/// # Errors
///
/// Returns an error if:
/// * The database file cannot be opened.
/// * The initial PRAGMA configurations fail.
/// * The schema initialization fails.
pub(crate) fn init_db(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;

    let journal_mode: String = conn.query_row("PRAGMA journal_mode = WAL", [], |r| r.get(0))?;
    if journal_mode != "wal" {
        anyhow::bail!(
            "Failed to switch to WAL mode. Current mode: {}",
            journal_mode
        );
    }

    conn.execute_batch(
        "
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;
        PRAGMA cache_size = -64000; -- Use 64MB of RAM for cache
    ",
    )?;

    conn.set_prepared_statement_cache_capacity(100);

    create_schema(&conn)?;

    Ok(conn)
}

/// Create the database schema.
///
/// This function creates the `artists`, `albums`, and `tracks` tables if they
/// do not already exist.
///
/// It also sets up:
///
/// * **Foreign Key Constraints**: Automated cleanup via `ON DELETE CASCADE`.
/// * **Performance Indices**: Indices on foreign keys to optimize join operations.
/// * **Uniqueness Constraints**: Prevention of duplicate artists, albums, or track files.
///
/// This operation is wrapped in a single SQL transaction to ensure the schema
/// is updated atomically.
///
/// # Errors
///
/// Returns an error if the transaction fails, if there are permission issues
/// with the database file, or if the SQL syntax is invalid.
fn create_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "BEGIN;

        CREATE TABLE IF NOT EXISTS artists (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL COLLATE NOCASE UNIQUE
        );

        CREATE TABLE IF NOT EXISTS albums (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            artist_id INTEGER NOT NULL,
            title TEXT NOT NULL COLLATE NOCASE,
            UNIQUE (artist_id, title),
            FOREIGN KEY (artist_id) REFERENCES artists (id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_albums_artist_id ON albums (artist_id);

        CREATE TABLE IF NOT EXISTS tracks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            durable_id INTEGER NOT NULL UNIQUE,
            album_id INTEGER NOT NULL,
            track_number INTEGER,
            title TEXT NOT NULL COLLATE NOCASE,
            duration INTEGER NOT NULL,
            genre TEXT,
            year INTEGER,
            filename TEXT NOT NULL UNIQUE,
            UNIQUE (album_id, filename),
            FOREIGN KEY (album_id) REFERENCES albums (id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_tracks_album_id ON tracks (album_id);

        CREATE TABLE IF NOT EXISTS track_stats (
            durable_id INTEGER PRIMARY KEY,
            play_count INTEGER NOT NULL DEFAULT 0,
            rating INTEGER NOT NULL DEFAULT 0
        );

        COMMIT;",
    )
    .context("Failed to create schema")
}

/// Fetches all artist names from the database, sorted alphabetically.
///
/// This function retrieves the complete list of artists available in the
/// library.
///
/// # Arguments
///
/// * `conn` - A reference to the SQLite connection.
///
/// # Errors
///
/// Returns an error if the SQL query fails or if there is a type mismatch
/// when mapping the database rows to the [`Artist`] struct.
///
/// # Examples
///
/// ```ignore
/// let artists = fetch_artist_names(&conn).expect("Failed to fetch artists");
/// assert!(!artists.is_empty());
/// ```
pub(crate) fn fetch_artist_names(conn: &Connection) -> Result<Vec<Artist>> {
    let mut stmt = conn.prepare_cached("SELECT id, name FROM artists ORDER BY name")?;
    let rows = stmt.query_map([], |row| {
        Ok(Artist {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
}

/// Fetches all album titles for a given artist from the database, sorted
/// alphabetically.
///
/// This function retrieves the complete list of albums available in the
/// library.
///
/// # Arguments
///
/// * `conn` - A reference to the SQLite connection.
///
/// # Errors
///
/// Returns an error if the SQL query fails or if there is a type mismatch
/// when mapping the database rows to the [`Album`] struct.
///
/// # Examples
///
/// ```ignore
/// let albums = fetch_album_titles(&conn, album_id).expect("Failed to fetch albums");
/// assert!(!albums.is_empty());
/// ```
pub(crate) fn fetch_artist_album_titles(conn: &Connection, artist_id: i32) -> Result<Vec<Album>> {
    let mut stmt =
        conn.prepare_cached("SELECT id, title, artist_id FROM albums WHERE artist_id = ?")?;
    let rows = stmt.query_map([artist_id], |row| {
        Ok(Album {
            id: row.get(0)?,
            title: row.get(1)?,
            artist_id: row.get(2)?,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
}

/// Fetches all tracks associated with a specific album, ordered by track
/// number and title.
///
/// This function retrieves the complete list of tracks for the given album
/// to facilitate sequential playback or detailed album views.
///
/// # Arguments
///
/// * `conn` - A reference to the SQLite connection.
/// * `album_id` - The unique identifier of the album.
///
/// # Errors
///
/// Returns an error if the SQL query fails or if there is a type mismatch
/// when mapping the database rows to the [`Track`] struct.
///
/// # Examples
///
/// ```ignore
/// let tracks = fetch_album_tracks(&conn, album_id).expect("Failed to fetch tracks");
/// assert!(!tracks.is_empty());
/// ```
pub fn fetch_album_tracks(conn: &Connection, album_id: i32) -> Result<Vec<Track>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, track_number, title, album_id, filename
         FROM tracks
         WHERE album_id = ?
         ORDER BY track_number, title
    ",
    )?;

    let rows = stmt.query_map([album_id], |row| {
        Ok(Track {
            id: row.get(0)?,
            track_number: row.get(1)?,
            title: row.get(2)?,
            album_id: row.get(3)?,
            filename: row.get(4)?,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Fetches all track information for an artist.
///
/// This function looks up all tracks for the artist with the given id by
/// joining the `tracks`, `albums` and `artists` tables.
///
/// # Arguments
///
/// * `conn` - A reference to the SQLite connection.
/// * `artist_id` - A unique identifier for the artist.
///
/// # Errors
///
/// Returns a [`rusqlite::Error`] if the database connection fails or the query
/// is malformed.
///
/// # Panics
///
/// This function will panic if the database connection has been poisoned.
///
/// # Examples
///
/// ```ignore
/// let tracks = fetch_artist_tracks(&conn, artist_id).expect("Failed to fetch tracks");
/// assert!(!tracks.is_empty());
/// ```
pub(crate) fn fetch_artist_trackinfo(conn: &Connection, artist_id: i32) -> Result<Vec<TrackInfo>> {
    let sql = "
        SELECT ar.name, al.title, tr.id, tr.track_number, tr.title, tr.filename
        FROM tracks tr
        JOIN albums al ON tr.album_id = al.id
        JOIN artists ar ON al.artist_id = ar.id
        WHERE ar.id = ?
        ORDER BY al.title, tr.track_number
    ";

    let mut stmt = conn.prepare_cached(sql)?;
    let results = stmt
        .query_map([artist_id], TrackInfo::from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}

/// Fetches all track information for an album.
///
/// This function looks up all tracks for the album with the given id by
/// joining the `tracks`, `albums` and `artists` tables.
///
/// # Arguments
///
/// * `conn` - A reference to the SQLite connection.
/// * `album_id` - A unique identifier for the album.
///
/// # Errors
///
/// Returns a [`rusqlite::Error`] if the database connection fails or the query
/// is malformed.
///
/// # Panics
///
/// This function will panic if the database connection has been poisoned.
///
/// # Examples
///
/// ```ignore
/// let tracks = fetch_album_tracks(&conn, album_id).expect("Failed to fetch tracks");
/// assert!(!tracks.is_empty());
/// ```
pub(crate) fn fetch_album_track_info(conn: &Connection, album_id: i32) -> Result<Vec<TrackInfo>> {
    let sql = "
        SELECT ar.name, al.title, tr.id, tr.track_number, tr.title, tr.filename
        FROM tracks tr
        JOIN albums al ON tr.album_id = al.id
        JOIN artists ar ON al.artist_id = ar.id
        WHERE al.id = ?
        ORDER BY tr.track_number
    ";

    let mut stmt = conn.prepare_cached(sql)?;
    let results = stmt
        .query_map([album_id], TrackInfo::from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}

/// Fetches track information.
///
/// This function looks up the track with the given id by joining the `tracks`,
/// `albums` and `artists` tables.
///
/// Note: A [`Vec`] is returned for consistency with other fetch functions,
/// though it will contain at most one element.
///
/// # Arguments
///
/// * `conn` - A reference to the SQLite connection.
/// * `track_id` - A unique identifier for the track.
///
/// # Errors
///
/// Returns a [`rusqlite::Error`] if the database connection fails or the query
/// is malformed.
///
/// # Panics
///
/// This function will panic if the database connection has been poisoned.
///
/// # Examples
///
/// ```ignore
/// let tracks = fetch_track(&conn, track_id).expect("Failed to fetch track");
/// ```
pub(crate) fn fetch_track_info(conn: &Connection, track_id: i32) -> Result<TrackInfo> {
    let sql = "
        SELECT ar.name, al.title, tr.id, tr.durable_id, tr.track_number, tr.title, tr.duration, tr.year, tr.genre, tr.filename
        FROM tracks tr
        JOIN albums al ON tr.album_id = al.id
        JOIN artists ar ON al.artist_id = ar.id
        WHERE tr.id = ?
    ";

    let mut stmt = conn.prepare_cached(sql)?;
    let result = stmt.query_one([track_id], TrackInfo::from_row)?;

    Ok(result)
}

pub(crate) fn search(conn: &Connection, query: &SearchQuery) -> Result<Vec<TrackInfo>> {
    let mut sql = String::from("
        SELECT ar.name, al.title, tr.id, tr.durable_id, tr.track_number, tr.title, tr.duration, tr.year, tr.genre, tr.filename
        FROM tracks tr
        JOIN albums al ON tr.album_id = al.id
        JOIN artists ar ON al.artist_id = ar.id
    ");

    let mut filters = Vec::new();
    let mut params = Vec::new();

    if query.search.len() >= MIN_SEARCH_LEN {
        filters.push("(ar.name LIKE ? OR al.title LIKE ? OR tr.title LIKE ?)".to_string());
        let param = format!("%{}%", query.search);
        params.push(param.clone());
        params.push(param.clone());
        params.push(param.clone());
    }

    if query.artist.len() >= MIN_SEARCH_LEN {
        filters.push("(ar.name LIKE ?)".to_string());
        params.push(format!("%{}%", query.artist.to_lowercase()));
    }

    if query.album.len() >= MIN_SEARCH_LEN {
        filters.push("(al.title LIKE ?)".to_string());
        params.push(format!("%{}%", query.album.to_lowercase()));
    }

    if query.track.len() >= MIN_SEARCH_LEN {
        filters.push("(tr.title LIKE ?)".to_string());
        params.push(format!("%{}%", query.track.to_lowercase()));
    }

    if !filters.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&filters.join(" AND "));
    }

    sql.push_str(" ORDER BY ar.name, al.title, tr.track_number");

    let mut stmt = conn.prepare_cached(&sql)?;
    let results = stmt
        .query_map(rusqlite::params_from_iter(params), TrackInfo::from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}

pub(crate) fn increment_play_count(conn: &Connection, durable_id: i64) -> Result<()> {
    let sql = "
        INSERT INTO track_stats (durable_id, play_count)
        VALUES (?1, 1)
        ON CONFLICT (durable_id)
        DO UPDATE SET play_count = play_count + 1";

    let mut stmt = conn.prepare_cached(sql)?;
    stmt.execute(params![durable_id])?;

    Ok(())
}

pub(crate) fn update_rating(conn: &Connection, durable_id: i64, rating: Rating) -> Result<()> {
    let sql = "
        INSERT INTO track_stats (durable_id, rating)
        VALUES (?1, ?2)
        ON CONFLICT (durable_id)
        DO UPDATE SET rating = ?2";

    let mut stmt = conn.prepare_cached(sql)?;
    stmt.execute(params![durable_id, rating])?;

    Ok(())
}
