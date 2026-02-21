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

use anyhow::{Context, Result, anyhow};
use lofty::config::ParsingMode;
use lofty::probe::Probe;
use lofty::tag::ItemKey;
use lofty::{config::ParseOptions, prelude::*};
use rusqlite::{Connection, Transaction, params};
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::Write,
    path::Path,
    sync::mpsc::Sender,
    time::{Duration, Instant},
};
use walkdir::WalkDir;

use crate::events::{AppEvent, CatalogEvent};

/// Recursively scans a directory for MP3 files and synchronizes the database.
///
/// This performs a full library rebuild by clearing all existing records and
/// re-indexing the provided root directory. Metadata is extracted from file
/// tags and normalized via an internal cache to prevent duplicate entries.
///
/// # Arguments
///
/// * `conn` - A mutable reference to the SQLite database connection.
/// * `paths` - The names of to the directories containing the music library.
/// * `event_tx` - The channel to send application events to.
///
/// # Returns
///
/// Returns the total number of tracks successfully imported into the database.
///
/// # Errors
///
/// Returns an error if the transaction fails, if the directory is inaccessible,
/// or if database constraints are violated during insertion.
pub(crate) fn process_music_library(
    conn: &mut Connection,
    paths: &Vec<String>,
    event_tx: &Sender<AppEvent>,
) -> Result<i64> {
    event_tx.send(AppEvent::Catalog(CatalogEvent::Started))?;

    let mut artist_cache: HashMap<String, i64> = HashMap::new();
    let mut album_cache: HashMap<(i64, String), i64> = HashMap::new();

    let tx = conn.transaction()?;

    tx.execute("DELETE FROM tracks", [])?;
    tx.execute("DELETE FROM albums", [])?;
    tx.execute("DELETE FROM artists", [])?;

    tx.execute(
        "DELETE FROM sqlite_sequence WHERE name IN ('artists', 'albums', 'tracks')",
        [],
    )?;

    let mut count = 0;

    let mut last_update = Instant::now();
    let update_interval = Duration::from_millis(100);

    let mut error_log = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("scan_log.txt")?;

    for root in paths {
        event_tx.send(AppEvent::Catalog(CatalogEvent::StartedDirectory(
            root.to_string(),
        )))?;

        for entry in WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext.to_ascii_lowercase() == "mp3")
            })
        {
            let path = entry.path();

            let track_result = process_track(&tx, path, &mut artist_cache, &mut album_cache);
            if let Err(e) = track_result {
                // let log_entry = format!("{} | {:#}\n", path.display(), e);
                // let log_entry = format!("{} | {:?}\n", path.display(), e);
                let log_entry = format!("{} | {:#}\n", path.display(), e);

                if let Err(write_err) = error_log.write_all(log_entry.as_bytes()) {
                    eprintln!("Critical: Could not write to error log file: {}", write_err);
                }
            }
            count += 1;

            if last_update.elapsed() >= update_interval {
                let filename = path
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "Unknown".into());

                let _ = event_tx.send(AppEvent::Catalog(CatalogEvent::ProcessedFile(
                    count, filename,
                )));

                last_update = Instant::now();
            }
        }

        event_tx.send(AppEvent::Catalog(CatalogEvent::FinishedDirectory(
            root.to_string(),
        )))?;
    }

    tx.commit().context("Failed to commit transaction")?;

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))?;

    event_tx.send(AppEvent::Catalog(CatalogEvent::Finished(count)))?;

    Ok(count)
}

fn process_track(
    tx: &Transaction,
    path: &Path,
    artist_cache: &mut HashMap<String, i64>,
    album_cache: &mut HashMap<(i64, String), i64>,
) -> Result<()> {
    let options = ParseOptions::new().parsing_mode(ParsingMode::Relaxed);

    let tagged_file = Probe::open(path)?.options(options).read()?;

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag())
        .ok_or_else(|| anyhow!("No tags found in file"))?;

    let artist_name = tag
        .get(ItemKey::AlbumArtist)
        .and_then(|item| item.value().text())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            tag.artist()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "Unknown Artist".into())
        });

    let album_title = tag
        .album()
        .map(|c| c.to_string())
        .unwrap_or_else(|| "Unknown Album".into());

    let track_title = tag.title().map(|c| c.to_string()).unwrap_or_else(|| {
        path.file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Unknown Track".into())
    });

    let year = tag.date().map(|ts| ts.year);
    let duration = i64::try_from(tagged_file.properties().duration().as_secs()).unwrap_or(-1);
    let genre = tag
        .genre()
        .map(|c| c.to_string())
        .unwrap_or_else(|| "".into());
    let track_number = tag.track();

    let durable_id = xxhash_rust::xxh3::xxh3_64(
        format!(
            "{}|{}|{}|{}",
            artist_name,
            album_title,
            track_number.unwrap_or_default(),
            track_title
        )
        .as_bytes(),
    ) as i64;

    let artist_id = if let Some(&id) = artist_cache.get(&artist_name) {
        id
    } else {
        tx.execute(
            "INSERT OR IGNORE INTO artists (name) VALUES (?)",
            params![artist_name],
        )?;
        let id: i64 = tx.query_row(
            "SELECT id FROM artists WHERE name = ?",
            params![artist_name],
            |r| r.get(0),
        )?;
        artist_cache.insert(artist_name, id);
        id
    };

    let album_key = (artist_id, album_title.clone());
    let album_id = if let Some(&id) = album_cache.get(&album_key) {
        id
    } else {
        tx.execute(
            "INSERT OR IGNORE INTO albums (artist_id, title) VALUES (?, ?)",
            params![artist_id, album_title],
        )?;
        let id: i64 = tx.query_row(
            "SELECT id FROM albums WHERE artist_id = ? AND title = ?",
            params![artist_id, album_title],
            |r| r.get(0),
        )?;
        album_cache.insert(album_key, id);
        id
    };

    let filename = path
        .to_str()
        .context("Path contains invalid UTF-8")?
        .to_string();
    tx.execute(
        "INSERT OR IGNORE INTO tracks (album_id, durable_id, track_number, title, duration, genre, year, filename) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        params![album_id, durable_id, track_number, track_title, duration, genre, year, filename],
    )?;

    tx.execute(
        "DELETE FROM track_stats WHERE durable_id NOT IN (SELECT durable_id FROM tracks)",
        [],
    )?;

    Ok(())
}
