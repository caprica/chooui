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

//! Media library indexing and database persistence.
//!
//! This module handles the discovery of audio files on the local filesystem
//! and the management of associated database records.
//!
//! It utilizes `WalkDir` for efficient directory traversal and `Lofty` for
//! metadata extraction.
//!
//! # Database Schema
//!
//! The library is organized into a three-tier hierarchy stored in SQLite:
//! 1. **Artists**: Unique performers or composers.
//! 2. **Albums**: Collections of tracks associated with an artist.
//! 3. **Tracks**: Individual audio files with paths and metadata.
//!
//! # Performance
//!
//! Large library scans are performed within an atomic SQLite transaction to
//! maximize write throughput and ensure database integrity. Internal caching
//! is used during processing to minimize redundant database lookups for
//! existing artist and album entries.

use anyhow::{Context, Result};
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::ItemKey;
use rusqlite::{params, Connection};
use walkdir::WalkDir;
use std::collections::HashMap;
use std::path::Path;

/// Recursively scans a directory for MP3 files and synchronizes the database.
///
/// This performs a full library rebuild by clearing all existing records and
/// re-indexing the provided root directory. Metadata is extracted from file
/// tags and normalized via an internal cache to prevent duplicate entries.
///
/// # Arguments
///
/// * `conn` - A mutable reference to the SQLite database connection.
/// * `root` - The filesystem path to the directory containing the music library.
///
/// # Returns
///
/// Returns the total number of tracks successfully imported into the database.
///
/// # Errors
///
/// Returns an error if the transaction fails, if the directory is inaccessible,
/// or if database constraints are violated during insertion.
pub(crate) fn process_music_library(conn: &mut Connection, root: &Path) -> Result<i64> {
    let mut artist_cache: HashMap<String, i64> = HashMap::new();
    let mut album_cache: HashMap<(i64, String), i64> = HashMap::new();

    let tx = conn.transaction()?;

    tx.execute("DELETE FROM tracks", [])?;
    tx.execute("DELETE FROM albums", [])?;
    tx.execute("DELETE FROM artists", [])?;

    tx.execute("DELETE FROM sqlite_sequence WHERE name IN ('artists', 'albums', 'tracks')", [])?;

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "mp3"))
    {
        let path = entry.path();

        let tagged_file = match Probe::open(path).and_then(|p| p.read()) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Skipping {:?}: {}", path, e);
                continue;
            }
        };

        let tag = match tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
            Some(t) => t,
            None => {
                println!("Skipping (no tags): {}", path.display());
                continue;
            }
        };

        let mut artist_name = tag.artist().unwrap_or_else(|| "Unknown Artist".into()).to_string();
        let album_title = tag.album().unwrap_or_else(|| "Unknown Album".into()).to_string();
        let year = tag.year();
        let track_title = tag.title().unwrap_or_else(|| path.file_name().unwrap().to_string_lossy()).to_string();
        let duration = i64::try_from(tagged_file.properties().duration().as_secs()).unwrap_or(-1);
        let genre = tag.genre().unwrap_or_else(|| "".into()).to_string();

        let album_artist_name = tag.get(&ItemKey::AlbumArtist)
            .and_then(|item| item.value().text())
            .map(|s| s.to_string());

        let display_artist = album_artist_name.as_deref().or(Some(&artist_name)).unwrap().to_string();
        artist_name = display_artist;

        let track_number: Option<u32> = tag.track();

        let artist_id = if let Some(&id) = artist_cache.get(&artist_name) {
            id
        } else {
            tx.execute("INSERT OR IGNORE INTO artists (name) VALUES (?)", params![artist_name])?;
            let id: i64 = tx.query_row("SELECT id FROM artists WHERE name = ?", params![artist_name], |r| r.get(0))?;
            artist_cache.insert(artist_name.clone(), id);
            id
        };

        let album_key = (artist_id, album_title.clone());
        let album_id = if let Some(&id) = album_cache.get(&album_key) {
            id
        } else {
            tx.execute("INSERT OR IGNORE INTO albums (artist_id, title) VALUES (?, ?)", params![artist_id, album_title])?;
            let id: i64 = tx.query_row("SELECT id FROM albums WHERE artist_id = ? AND title = ?", params![artist_id, album_title], |r| r.get(0))?;
            album_cache.insert(album_key, id);
            id
        };

        let filename = path.to_str().context("Path contains invalid UTF-8")?.to_string();

        tx.execute(
            "INSERT OR IGNORE INTO tracks (album_id, track_number, title, duration, genre, year, filename) VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![album_id, track_number, track_title, duration, genre, year, filename],
        )?;
    }

    tx.commit().context("Failed to commit transaction")?;

    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM tracks",
        [],
        |row| row.get(0)
    )?;

    Ok(count)
}
