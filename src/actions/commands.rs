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

//! Asynchronous application command processing.
//!
//! This module implements the command pattern used to offload potentially
//! blocking database queries from the main UI thread. It provides a dedicated
//! worker loop that translates [`AppCommand`] requests into database
//! operations //! and broadcasts the results back to the application via
//! [`AppEvent`]s.

use std::{path::Path, sync::mpsc::{Receiver, Sender}, thread};
use anyhow::Result;
use rusqlite::Connection;

use crate::{actions::events::AppEvent, db::{self, scan}};

const DATABASE_FILE: &str = "music.db";

#[derive(Debug)]
pub(crate) enum AppCommand {
    ScanCatalog,
    GetBrowserArtists,
    GetBrowserAlbums(i32),
    GetBrowserTracks(i32),
    GetNowPlaying(i32),
    AddArtistToQueue(i32),
    AddAlbumToQueue(i32),
    AddTrackToQueue(i32),
}

/// Spawns a background thread to process application commands.
///
/// This worker thread initializes its own database connection and enters
/// a blocking loop, listening for incoming [`AppCommand`]s.
///
/// # Arguments
///
/// * `command_rx` - The receiving end of the command channel.
/// * `event_tx` - The sending end of the channel for broadcasting results.
pub(crate) fn spawn_command_worker(command_rx: Receiver<AppCommand>, event_tx: Sender<AppEvent>) {
    thread::spawn(move || {
        let mut conn = db::init_db(DATABASE_FILE).expect("Failed to initialise database");

        while let Ok(request) = command_rx.recv() {
            if let Err(e) = handle_command(&mut conn, request, &event_tx) {
                let _ = event_tx.send(AppEvent::Error(e.to_string()));
            }
        }
    });
}

/// Orchestrates the execution of a single command.
///
/// This function implements the logic for each command and sends the result
/// back through the application event channel.
fn handle_command(conn: &mut Connection, command: AppCommand, event_tx: &Sender<AppEvent>) -> Result<()> {
    match command {
        AppCommand::ScanCatalog => {
            event_tx.send(AppEvent::SetBrowserArtists(vec![]))?;
            event_tx.send(AppEvent::SetBrowserAlbums(vec![]))?;
            event_tx.send(AppEvent::SetBrowserTracks(vec![]))?;

            let music_dir = "/disks/data/othermusic"; // FIXME hard-code
            scan::process_music_library(conn, Path::new(music_dir))?;

            event_tx.send(AppEvent::CatalogUpdated)?;
        },
        AppCommand::GetBrowserArtists => {
            let artists = db::fetch_artist_names(&conn)?;
            event_tx.send(AppEvent::SetBrowserArtists(artists))?;
        }
        AppCommand::GetBrowserAlbums(artist_id) => {
            let albums = db::fetch_artist_album_titles(&conn, artist_id)?;
            event_tx.send(AppEvent::SetBrowserAlbums(albums))?;
        }
        AppCommand::GetBrowserTracks(album_id) => {
            let tracks = db::get_album_tracks_x(&conn, album_id)?;
            event_tx.send(AppEvent::SetBrowserTracks(tracks))?;
        }
        AppCommand::GetNowPlaying(track_id) => {
            let track_info = db::get_track_info(&conn, track_id)?;
            event_tx.send(AppEvent::SetNowPlaying(track_info))?;
        },
        AppCommand::AddArtistToQueue(artist_id) => {
            let tracks = db::fetch_artist_tracks(&conn, artist_id)?;
            event_tx.send(AppEvent::AddTracksToQueue(tracks))?;
        },
        AppCommand::AddAlbumToQueue(album_id) => {
            let tracks = db::fetch_album_tracks(&conn, album_id)?;
            event_tx.send(AppEvent::AddTracksToQueue(tracks))?;
        },
        AppCommand::AddTrackToQueue(track_id) => {
            let tracks = db::fetch_track(&conn, track_id)?;
            event_tx.send(AppEvent::AddTracksToQueue(tracks))?;
        }
    }

    Ok(())
}
