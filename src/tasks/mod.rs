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

//! Asynchronous application task processing.
//!
//! This module implements the command pattern used to offload tasks such as
//! potentially blocking database queries from the main UI thread. It provides
//! a dedicated worker loop that translates [`AppTask`] requests into database
//! (and other operations and broadcasts the results back to the application
//! via [`AppEvent`]s.
//!
//! Only actions that may block, or may take more than a trivial amount of time
//! to process, should be implemented as tasks. Other actions are likely more
//! suited to by events.

mod handlers;
use handlers::*;

use anyhow::Result;
use rusqlite::Connection;
use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
};

use crate::{
    config::AppConfig,
    db::{self},
    events::AppEvent,
    model::{Rating, SearchQuery, TrackInfo},
};

const DATABASE_FILE: &str = "music.db";

#[derive(Debug)]
pub(crate) enum AppTask {
    ScanCatalog,

    Search(SearchQuery),

    GetBrowserArtists,
    GetBrowserAlbums(i32),
    GetBrowserTracks(i32),

    AddArtistToQueue(i32),
    AddAlbumToQueue(i32),
    AddTrackToQueue(i32),
    AddMatchingArtistToQueue(String),
    AddMatchingAlbumToQueue(String),
    AddMatchingTrackToQueue(String),

    PlayTrack(TrackInfo),
    RateTrack(TrackInfo, Rating),
}

/// Bundles shared resources required by task handlers to simplify resource
/// passing when invoking those handler functions.
struct TaskContext<'a> {
    config: &'a AppConfig,
    event_tx: &'a Sender<AppEvent>,
    conn: &'a mut Connection,
}

/// Spawns a background thread to process application tasks.
///
/// This worker thread initializes its own database connection and enters
/// a blocking loop, listening for incoming [`AppTask`]s.
///
/// # Arguments
///
/// * `config` - The application configuration.
/// * `command_rx` - The receiving end of the command channel.
/// * `event_tx` - The sending end of the channel for broadcasting results.
pub(crate) fn spawn_task_worker(
    config: &AppConfig,
    command_rx: Receiver<AppTask>,
    event_tx: Sender<AppEvent>,
) {
    let config = config.clone(); // maybe the caller should clone this

    thread::spawn(move || {
        let mut conn = db::init_db(DATABASE_FILE).expect("Failed to initialise database");

        while let Ok(task) = command_rx.recv() {
            let mut ctx = TaskContext {
                config: &config,
                event_tx: &event_tx,
                conn: &mut conn,
            };

            if let Err(e) = handle_task(task, &mut ctx) {
                let _ = event_tx.send(AppEvent::Error(e.to_string()));
            }
        }
    });
}

/// Orchestrates the execution of a single task.
///
/// This function implements the logic for each task and sends the result back
/// through the application event channel.
fn handle_task(task: AppTask, ctx: &mut TaskContext) -> Result<()> {
    match task {
        AppTask::ScanCatalog => scan_catalog(ctx),

        AppTask::Search(query) => search(ctx, query),

        AppTask::GetBrowserArtists => get_browser_artists(ctx),
        AppTask::GetBrowserAlbums(id) => get_browser_albums(ctx, id),
        AppTask::GetBrowserTracks(id) => get_browser_tracks(ctx, id),

        AppTask::AddArtistToQueue(id) => add_artist_to_queue(ctx, id),
        AppTask::AddAlbumToQueue(id) => add_album_to_queue(ctx, id),
        AppTask::AddTrackToQueue(id) => add_track_to_queue(ctx, id),
        AppTask::AddMatchingArtistToQueue(artist) => Ok(()),
        AppTask::AddMatchingAlbumToQueue(album) => Ok(()),
        AppTask::AddMatchingTrackToQueue(track) => Ok(()),

        AppTask::PlayTrack(track) => play_track(ctx, track),
        AppTask::RateTrack(track, rating) => rate_track(ctx, track, rating),
    }
}
