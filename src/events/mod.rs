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

//! Application logic, event handling, and command dispatching.
//!
//! This module acts as the central hub for the "Controller" logic of the
//! application. It organizes how various inputs are translated into internal
//! state changes.
//!
//! # Organization
//!
//! * [`events`]: Defines the raw input types (keyboard, media player, tick
//!   events).
//! * [`commands`]: Contains high-level application commands (add to queue and
//!   so on).
//!
//! All public members of sub-modules are re-exported at this level for
//! convenient access.

mod handlers;
mod key_handlers;

use handlers::*;
use key_handlers::*;

use std::io::Stdout;

use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::{Terminal, prelude::CrosstermBackend};

use crate::{
    App, MainView,
    model::{Album, Artist, SearchQuery, Track, TrackInfo},
    player::PlayerState,
    render::draw,
};

#[derive(Debug)]
pub(crate) enum AppEvent {
    Key(KeyEvent),

    Catalog(CatalogEvent),
    CatalogUpdated,

    SetMainView(MainView),

    PlayTrack(TrackInfo),
    PlayPlaylist,

    AddTracksToPlaylist(Vec<TrackInfo>),
    AddSelectionToPlaylist,

    NewSearchQuery(SearchQuery),
    SearchResultsReady(Vec<TrackInfo>),

    ArtistSelectionChanged(i32),
    AlbumSelectionChanged(i32),
    TrackSelectionChanged(i32),

    SetBrowserArtists(Vec<Artist>),
    SetBrowserAlbums(Vec<Album>),
    SetBrowserTracks(Vec<Track>),

    SetNowPlaying(TrackInfo),

    AddTracksToQueue(Vec<TrackInfo>),

    PlayerStateChanged(PlayerState),
    TitleChanged(String),
    DurationChanged(u64),
    TimeChanged(f64),
    VolumeChanged(u32),
    TrackFinished,

    Tick,

    ExitApplication,

    Error(String),
    FatalError(String),

    AddMatchingArtistToQueue(String),
    AddMatchingAlbumToQueue(String),
    AddMatchingTrackToQueue(String),

    AddSelectedArtistToQueue,
    AddSelectedAlbumToQueue,
    AddSelectedTrackToQueue,

    ClearQueue,
}

#[derive(Debug)]
pub(crate) enum CatalogEvent {
    Started,
    StartedDirectory(String),
    ProcessedFile(usize, String),
    FinishedDirectory(String),
    Finished(i64),
}

/// Runs the main application loop, handling events and rendering the UI in the
/// terminal.
///
/// This function loops until a 'quit' event is received or the event channel
/// is closed.
pub(crate) fn process_events(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> Result<()> {
    while let Ok(event) = app.event_rx.recv() {
        if matches!(event, AppEvent::ExitApplication) {
            break;
        }

        match event {
            AppEvent::Key(key) => process_key_event(app, key)?,
            AppEvent::Catalog(catalog_event) => handle_catalog_event(app, catalog_event),
            AppEvent::CatalogUpdated => handle_catalog_updated(app),
            AppEvent::SetMainView(view) => handle_set_main_view(app, view),
            AppEvent::NewSearchQuery(q) => handle_new_search_query(app, q)?,
            AppEvent::SearchResultsReady(res) => handle_search_results_ready(app, res)?,
            AppEvent::AddSelectionToPlaylist => handle_add_selection_to_playlist(app),
            AppEvent::PlayTrack(track) => handle_play_track(app, track)?,
            AppEvent::PlayPlaylist => handle_play_playlist(app)?,
            AppEvent::AddTracksToPlaylist(tracks) => handle_add_tracks_to_playlist(app, tracks)?,
            AppEvent::ArtistSelectionChanged(id) => handle_artist_selection_changed(app, id)?,
            AppEvent::AlbumSelectionChanged(id) => handle_album_selection_changed(app, id)?,
            AppEvent::AddTracksToQueue(tracks) => handle_add_tracks_to_queue(app, tracks),
            AppEvent::SetBrowserArtists(artists) => handle_set_browser_artists(app, artists)?,
            AppEvent::SetBrowserAlbums(albums) => handle_set_browser_albums(app, albums)?,
            AppEvent::SetBrowserTracks(tracks) => handle_set_browser_tracks(app, tracks)?,
            AppEvent::SetNowPlaying(info) => handle_set_now_playing(app, info),
            AppEvent::PlayerStateChanged(state) => handle_player_state_changed(app, state),
            AppEvent::TitleChanged(title) => handle_title_changed(app, title),
            AppEvent::DurationChanged(duration) => handle_duration_changed(app, duration),
            AppEvent::VolumeChanged(volume) => handle_volume_changed(app, volume),
            AppEvent::TrackFinished => handle_track_finished(app)?,
            AppEvent::TimeChanged(secs) => handle_time_changed(app, secs),
            AppEvent::AddSelectedArtistToQueue => handle_add_selected_artist_to_queue(app),
            AppEvent::AddSelectedAlbumToQueue => handle_add_selected_album_to_queue(app),
            AppEvent::AddSelectedTrackToQueue => handle_add_selected_track_to_queue(app),
            AppEvent::ClearQueue => handle_clear_queue(app),
            AppEvent::Tick | _ => handle_tick(app),
        }

        terminal.draw(|f| draw(f, app))?;
    }
    Ok(())
}
