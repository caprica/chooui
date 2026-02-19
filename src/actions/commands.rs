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
//! worker loop that translates [`AppCommand`] requests into database (and
//! other operations and broadcasts the results back to the application via
//! [`AppEvent`]s.

use anyhow::Result;
use rusqlite::Connection;
use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
};

use crate::{
    MainView,
    actions::events::AppEvent,
    config::AppConfig,
    db::{self, scan},
    model::{Rating, SearchQuery, TrackInfo},
};

const DATABASE_FILE: &str = "music.db";

const MIN_SEARCH_LEN: usize = 3;

#[derive(Debug)]
pub(crate) enum AppCommand {
    ScanCatalog,
    SetMainView(MainView),
    Search(SearchQuery),
    AddSelectionToPlaylist,
    GetBrowserArtists,
    GetBrowserAlbums(i32),
    GetBrowserTracks(i32),
    AddArtistToQueue(i32),
    AddAlbumToQueue(i32),
    AddTrackToQueue(i32),
    PlayTrack(TrackInfo),

    PlayPlaylist,
    RateTrack(TrackInfo, Rating),
    ExitApplication,
    AddMatchingArtistToQueue(String),
    AddMatchingAlbumToQueue(String),
    AddMatchingTrackToQueue(String),
    AddSelectedArtistToQueue,
    AddSelectedAlbumToQueue,
    AddSelectedTrackToQueue,
}

/// Spawns a background thread to process application commands.
///
/// This worker thread initializes its own database connection and enters
/// a blocking loop, listening for incoming [`AppCommand`]s.
///
/// # Arguments
///
/// * `config` - The application configuration.
/// * `command_rx` - The receiving end of the command channel.
/// * `event_tx` - The sending end of the channel for broadcasting results.
pub(crate) fn spawn_command_worker(
    config: &AppConfig,
    command_rx: Receiver<AppCommand>,
    event_tx: Sender<AppEvent>,
) {
    let config = config.clone();

    thread::spawn(move || {
        let mut conn = db::init_db(DATABASE_FILE).expect("Failed to initialise database");

        while let Ok(request) = command_rx.recv() {
            if let Err(e) = handle_command(&config, &mut conn, request, &event_tx) {
                let _ = event_tx.send(AppEvent::Error(e.to_string()));
            }
        }
    });
}

/// Orchestrates the execution of a single command.
///
/// This function implements the logic for each command and sends the result
/// back through the application event channel.
fn handle_command(
    config: &AppConfig,
    conn: &mut Connection,
    command: AppCommand,
    event_tx: &Sender<AppEvent>,
) -> Result<()> {
    match command {
        AppCommand::ScanCatalog => {
            event_tx.send(AppEvent::SetBrowserArtists(vec![]))?;
            event_tx.send(AppEvent::SetBrowserAlbums(vec![]))?;
            event_tx.send(AppEvent::SetBrowserTracks(vec![]))?;

            let music_dirs = &config.media_dirs;
            scan::process_music_library(conn, music_dirs, event_tx)
                .expect("failed to process catalog");

            event_tx.send(AppEvent::CatalogUpdated)?;
        }
        AppCommand::SetMainView(main_view) => {
            event_tx.send(AppEvent::SetMainView(main_view))?;
        }
        AppCommand::AddSelectionToPlaylist => {
            event_tx.send(AppEvent::AddSelectionToPlaylist)?;
        }
        AppCommand::Search(query) => {
            let can_search = query.search.len() >= MIN_SEARCH_LEN
                || query.artist.len() >= MIN_SEARCH_LEN
                || query.album.len() >= MIN_SEARCH_LEN
                || query.track.len() >= MIN_SEARCH_LEN;
            if can_search {
                let search_results = db::search(&conn, &query)?;
                event_tx.send(AppEvent::SearchResultsReady(search_results))?;
            }
        }
        AppCommand::GetBrowserArtists => {
            let artists = db::fetch_artist_names(&conn)?;
            event_tx.send(AppEvent::SetBrowserArtists(artists))?;
        }
        AppCommand::GetBrowserAlbums(artist_id) => {
            let albums = db::fetch_artist_album_titles(&conn, artist_id)?;
            event_tx.send(AppEvent::SetBrowserAlbums(albums))?;
        }
        AppCommand::GetBrowserTracks(album_id) => {
            let tracks = db::fetch_album_tracks(&conn, album_id)?;
            event_tx.send(AppEvent::SetBrowserTracks(tracks))?;
        }
        AppCommand::AddArtistToQueue(artist_id) => {
            let tracks = db::fetch_artist_trackinfo(&conn, artist_id)?;
            event_tx.send(AppEvent::AddTracksToQueue(tracks))?;
        }
        AppCommand::AddAlbumToQueue(album_id) => {
            let tracks = db::fetch_album_track_info(&conn, album_id)?;
            event_tx.send(AppEvent::AddTracksToQueue(tracks))?;
        }
        AppCommand::AddTrackToQueue(track_id) => {
            let tracks = vec![db::fetch_track_info(&conn, track_id)?];
            event_tx.send(AppEvent::AddTracksToQueue(tracks))?;
        }
        AppCommand::PlayTrack(track) => {
            let durable_id = track.durable_id;

            event_tx.send(AppEvent::PlayTrack(track))?;

            // FIXME just integration testing here for now, probably we'd wait until after the first 5 seconds played
            db::increment_play_count(conn, durable_id)?;
        }
        AppCommand::PlayPlaylist => {
            event_tx.send(AppEvent::PlayPlaylist)?;
        }
        AppCommand::RateTrack(track, rating) => {
            // FIXME just integration testing here for now, it should probably become a toggle
            db::update_rating(conn, track.durable_id, rating)?;
        }
        AppCommand::ExitApplication => {
            event_tx.send(AppEvent::ExitApplication)?;
        }

        AppCommand::AddMatchingArtistToQueue(artist) => {}
        AppCommand::AddMatchingAlbumToQueue(album) => {}
        AppCommand::AddMatchingTrackToQueue(track) => {}

        AppCommand::AddSelectedArtistToQueue => {
            event_tx.send(AppEvent::AddSelectedArtistToQueue)?
        }
        AppCommand::AddSelectedAlbumToQueue => event_tx.send(AppEvent::AddSelectedAlbumToQueue)?,
        AppCommand::AddSelectedTrackToQueue => event_tx.send(AppEvent::AddSelectedTrackToQueue)?,
    }

    Ok(())
}
