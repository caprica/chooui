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

//! Application event distribution and orchestration.
//!
//! This module defines the central event-handling logic for the application,
//! bridging the gap between user input (keyboard), background worker updates
//! (database, audio player), and the UI rendering pipeline.
//!
//! # Architecture
//!
//! The system follows a reactive event-loop pattern:
//!
//! 1. **Capture**: Events are received via the [`AppEvent`] enum through an
//!    asynchronous channel.
//! 2. **Process**: The [`process_events`] function updates the [`App`] state,
//!    triggers commands to background workers (like the database or player),
//!    and manages UI navigation logic.
//! 3. **Render**: After each event is processed, the UI is re-drawn using the
//!   `ratatui` terminal.

use std::{io::Stdout, sync::mpsc::Sender};

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{Terminal, prelude::CrosstermBackend};

use crate::{
    App, MainView, PlayMode, RepeatMode,
    actions::commands::AppCommand,
    browser::MediaBrowserPane,
    db,
    model::{Album, Artist, SearchQuery, Track, TrackInfo},
    player::PlayerState,
    render::draw,
};

const FINE_VOLUME_DELTA: i32 = 1;
const VOLUME_DELTA: i32 = 5;

const FINE_SEEK_DELTA: i32 = 5;
const SEEK_DELTA: i32 = 20;

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
}

#[derive(Debug)]
pub(crate) enum CatalogEvent {
    Started,
    StartedDirectory(String),
    ProcessedFile(usize, String),
    FinishedDirectory(String),
    Finished(i64),
}

pub(crate) trait AppEventProcessor {
    fn process_event(&mut self, event: Event, event_tx: &Sender<AppEvent>) -> Result<()>;
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

            AppEvent::Catalog(catalog_event) => {
                match catalog_event {
                    CatalogEvent::Started => app.catalog.prepare_scan(&app.config.media_dirs),
                    CatalogEvent::StartedDirectory(dir) => app.catalog.begin_scan_directory(&dir),
                    CatalogEvent::ProcessedFile(count, filename) => {
                        app.catalog.update_scan_directory(count)
                    }
                    CatalogEvent::FinishedDirectory(dir) => app.catalog.end_scan_directory(),
                    CatalogEvent::Finished(count) => app.catalog.finish_scan(),
                };
            }

            AppEvent::CatalogUpdated => {
                // Ensure the browser view is updated when the catalog is updated (might change in future to just load on-demand in the browser)
                app.command_tx.send(AppCommand::GetBrowserArtists).unwrap();
            }

            AppEvent::SetMainView(main_view) => {
                match main_view {
                    MainView::Playlist => {
                        app.playlist_view.is_active = true;

                        app.catalog_view.is_active = false;
                        app.favourites_view.is_active = false;
                        app.search_view.is_active = false;
                    }
                    MainView::Search => {
                        app.search_view.is_active = true;

                        app.catalog_view.is_active = false;
                        app.favourites_view.is_active = false;
                        app.playlist_view.is_active = false;
                    }
                    MainView::Favourites => {
                        app.favourites_view.is_active = true;

                        app.catalog_view.is_active = false;
                        app.playlist_view.is_active = false;
                        app.search_view.is_active = false;
                    }
                    MainView::Browse => {
                        // FIXME

                        app.favourites_view.is_active = false;
                        app.catalog_view.is_active = false;
                        app.playlist_view.is_active = false;
                        app.search_view.is_active = false;
                    }
                    MainView::Catalog => {
                        app.catalog_view.is_active = true;

                        app.favourites_view.is_active = false;
                        app.playlist_view.is_active = false;
                        app.search_view.is_active = false;
                    }
                }
                app.main_view = main_view;
            }

            AppEvent::NewSearchQuery(query) => app.command_tx.send(AppCommand::Search(query))?,
            AppEvent::SearchResultsReady(results) => {
                app.search.set_tracks(results);

                app.search_view.track_table.reset_table_selection();
                app.command_tx
                    .send(AppCommand::SetMainView(MainView::Search))?;
            }

            AppEvent::AddSelectionToPlaylist => {
                let tracks = app.search_view.track_table.clone_selected_tracks();
                app.queue.add_tracks(tracks);
            }

            AppEvent::PlayTrack(track) => {
                app.play_mode = PlayMode::PlayOne;
                app.audio_player.play_file(&track.filename)?;
                app.now_playing = Some(track);
            }

            AppEvent::PlayPlaylist => {
                app.play_mode = PlayMode::Playlist;

                match app.current_queue_idx {
                    Some(idx) => {}
                    None => {
                        let lock = app.queue.tracks();
                        let tracks = lock.lock().unwrap();
                        if !tracks.is_empty() {
                            app.current_queue_idx = Some(0);
                            if let Some(track) = tracks.get(0).cloned() {
                                app.audio_player.play_file(&track.filename)?;
                                app.now_playing = Some(track);
                            }
                        }
                    }
                }
            }

            AppEvent::AddTracksToPlaylist(tracks) => {
                app.queue.add_tracks(tracks);

                app.command_tx
                    .send(AppCommand::SetMainView(MainView::Playlist))?;
                app.search_view.track_table.clear_selection();
                app.playlist_view.track_table.ensure_table_selection();
            }

            AppEvent::ArtistSelectionChanged(id) => {
                app.command_tx.send(AppCommand::GetBrowserAlbums(id))?
            }
            AppEvent::AlbumSelectionChanged(id) => {
                app.command_tx.send(AppCommand::GetBrowserTracks(id))?
            }
            AppEvent::AddTracksToQueue(tracks) => app.queue.add_tracks(tracks),

            AppEvent::SetBrowserArtists(artists) => {
                let first_id = artists.first().map(|a| a.id);
                app.media_browser.set_artists(artists);
                if let Some(id) = first_id {
                    app.event_tx.send(AppEvent::ArtistSelectionChanged(id))?;
                }
            }
            AppEvent::SetBrowserAlbums(albums) => {
                let first_id = albums.first().map(|a| a.id);
                app.media_browser.set_albums(albums);
                if let Some(id) = first_id {
                    app.event_tx.send(AppEvent::AlbumSelectionChanged(id))?;
                }
            }
            AppEvent::SetBrowserTracks(tracks) => {
                let first_id = tracks.first().map(|t| t.id);
                app.media_browser.set_tracks(tracks);
                if let Some(id) = first_id {
                    app.event_tx.send(AppEvent::TrackSelectionChanged(id))?;
                }
            }

            // Player State
            AppEvent::SetNowPlaying(info) => app.now_playing = Some(info),
            AppEvent::PlayerStateChanged(state) => app.player_state = state,
            AppEvent::TitleChanged(title) => app.player_track_name = Some(title),
            AppEvent::DurationChanged(dur) => app.player_duration = Some(dur),
            AppEvent::VolumeChanged(vol) => app.volume = Some(vol),

            AppEvent::TrackFinished => {
                app.player_time = app.player_duration;

                if app.play_mode == PlayMode::Playlist {
                    let lock = app.queue.tracks();
                    let tracks = lock.lock().unwrap();
                    let total_tracks = tracks.len();

                    if total_tracks == 0 {
                        app.current_queue_idx = None;
                        return Ok(());
                    }

                    if let Some(idx) = app.current_queue_idx.as_mut() {
                        if app.repeat_mode == RepeatMode::RepeatOne {
                            // nothing
                        } else {
                            let next_idx = *idx + 1;
                            if next_idx < total_tracks {
                                *idx = next_idx;
                            } else if app.repeat_mode == RepeatMode::RepeatAll {
                                *idx = 0;
                            } else {
                                app.current_queue_idx = None;
                            }
                        }

                        if let Some(valid_idx) = app.current_queue_idx {
                            if let Some(track) = tracks.get(valid_idx) {
                                app.audio_player.play_file(&track.filename)?;
                                app.now_playing = Some((*track).clone());
                            }
                        } else {
                            app.now_playing = None;
                        }
                    }
                }
            }

            AppEvent::TimeChanged(seconds) => {
                app.player_time = Some(seconds as u64);
                if let Some(duration) = app.player_duration {
                    app.player_position = if duration > 0 {
                        Some(seconds / duration as f64)
                    } else {
                        None
                    };
                }
            }
            AppEvent::Tick => {} // _ => {}

            AppEvent::AddSelectedArtistToQueue => match app.main_view {
                MainView::Search => {
                    let tracks = app.search_view.track_table.clone_selected_artist_tracks();
                    app.queue.add_tracks(tracks);
                }
                MainView::Favourites => {}
                MainView::Browse => {}
                _ => {}
            },

            AppEvent::AddSelectedAlbumToQueue => match app.main_view {
                MainView::Search => {
                    let tracks = app.search_view.track_table.clone_selected_album_tracks();
                    app.queue.add_tracks(tracks);
                }
                MainView::Favourites => {}
                MainView::Browse => {}
                _ => {}
            },

            AppEvent::AddSelectedTrackToQueue => match app.main_view {
                MainView::Search => {
                    if let Some(track) = app.search_view.track_table.clone_selected_track() {
                        let tracks = vec![track];
                        app.queue.add_tracks(tracks);
                    }
                }
                MainView::Favourites => {}
                MainView::Browse => {}
                _ => {}
            },

            _ => {}
        }

        // Render after every event processed
        terminal.draw(|f| draw(f, app))?;
    }

    Ok(())
}

/// Maps keyboard input to application actions and playback commands.
///
/// This function acts as the primary input router for the TUI, translating
/// low-level [`KeyEvent`]s into high-level domain logic. It handles:
///
/// * **Application Control**: Life-cycle events like exiting the program.
/// * **Navigation**: Moving between artists, albums, and tracks in the media
///   browser.
/// * **Playback**: Controlling the audio engine (play, pause, seek, volume).
/// * **Library Management**: Adding items to the playback queue or clearing
///   it.
///
/// # Arguments
///
/// * `app` - A mutable reference to the application state.
/// * `key` - The key event captured from the terminal backend.
///
/// # Errors
///
/// Returns an error if a command fails to send to a background worker or if
/// a requested action cannot be executed.
fn process_key_event(app: &mut App, key: KeyEvent) -> Result<()> {
    let event = Event::Key(key);
    let handled = app
        .commander
        .handle_event(event.clone(), &mut app.command_tx);
    if handled {
        return Ok(());
    }

    if app.playlist_view.is_active {
        let event = Event::Key(key);
        app.playlist_view
            .process_event(event, &app.command_tx, &app.event_tx)?;
    }

    if app.search_view.is_active {
        let event = Event::Key(key);
        app.search_view
            .process_event(event, &app.command_tx, &app.event_tx)?;
    }

    process_global_key_event(app, key)?;
    Ok(())
}

fn process_global_key_event(app: &mut App, key: KeyEvent) -> Result<()> {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) => {
            app.event_tx.send(AppEvent::ExitApplication)?;
        }

        (KeyCode::Char('1'), _) => app
            .command_tx
            .send(AppCommand::SetMainView(MainView::Playlist))?,
        (KeyCode::Char('2'), _) => app
            .command_tx
            .send(AppCommand::SetMainView(MainView::Search))?,
        (KeyCode::Char('3'), _) => app
            .command_tx
            .send(AppCommand::SetMainView(MainView::Favourites))?,
        (KeyCode::Char('4'), _) => app
            .command_tx
            .send(AppCommand::SetMainView(MainView::Browse))?,
        (KeyCode::Char('5'), _) => app
            .command_tx
            .send(AppCommand::SetMainView(MainView::Catalog))?,

        // Navigation: Down / j
        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => match app.media_browser.active_pane {
            MediaBrowserPane::Artist => {
                app.media_browser.next_artist();
                if let Some(id) = app.media_browser.selected_artist_id() {
                    app.event_tx.send(AppEvent::ArtistSelectionChanged(id))?;
                }
            }
            MediaBrowserPane::Album => {
                app.media_browser.next_album();
                if let Some(id) = app.media_browser.selected_album_id() {
                    app.event_tx.send(AppEvent::AlbumSelectionChanged(id))?;
                }
            }
            MediaBrowserPane::Track => {
                app.media_browser.next_track();
                if let Some(id) = app.media_browser.selected_track_id() {
                    app.event_tx.send(AppEvent::TrackSelectionChanged(id))?;
                }
            }
        },

        // Navigation: Up / k
        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => match app.media_browser.active_pane {
            MediaBrowserPane::Artist => {
                app.media_browser.previous_artist();
                if let Some(id) = app.media_browser.selected_artist_id() {
                    app.event_tx.send(AppEvent::ArtistSelectionChanged(id))?;
                }
            }
            MediaBrowserPane::Album => {
                app.media_browser.previous_album();
                if let Some(id) = app.media_browser.selected_album_id() {
                    app.event_tx.send(AppEvent::AlbumSelectionChanged(id))?;
                }
            }
            MediaBrowserPane::Track => {
                app.media_browser.previous_track();
                if let Some(id) = app.media_browser.selected_track_id() {
                    app.event_tx.send(AppEvent::TrackSelectionChanged(id))?;
                }
            }
        },

        // Pane Navigation
        (KeyCode::Char('h'), _) | (KeyCode::Left, _) => app.media_browser.previous_pane(),
        (KeyCode::Char('l'), _) | (KeyCode::Right, _) => app.media_browser.next_pane(),

        (KeyCode::Char(','), _) => app.audio_player.seek(-FINE_SEEK_DELTA)?,
        (KeyCode::Char('.'), _) => app.audio_player.seek(FINE_SEEK_DELTA)?,
        (KeyCode::Char('<'), _) => app.audio_player.seek(-SEEK_DELTA)?,
        (KeyCode::Char('>'), _) => app.audio_player.seek(SEEK_DELTA)?,
        (KeyCode::Char(' '), _) => app.audio_player.toggle_pause()?,
        (KeyCode::Char('s'), _) => app.audio_player.stop()?,
        (KeyCode::Char('-'), _) => app.audio_player.adjust_volume(-FINE_VOLUME_DELTA)?,
        (KeyCode::Char('='), _) => app.audio_player.adjust_volume(FINE_VOLUME_DELTA)?,
        (KeyCode::Char('_'), _) => app.audio_player.adjust_volume(-VOLUME_DELTA)?,
        (KeyCode::Char('+'), _) => app.audio_player.adjust_volume(VOLUME_DELTA)?,
        (KeyCode::Char('m'), _) => app.audio_player.toggle_mute()?,

        // Queue Management
        (KeyCode::Char('a'), _) => match app.media_browser.active_pane {
            MediaBrowserPane::Artist => {
                if let Some(id) = app.media_browser.selected_artist_id() {
                    app.command_tx.send(AppCommand::AddArtistToQueue(id))?;
                }
            }
            MediaBrowserPane::Album => {
                if let Some(id) = app.media_browser.selected_album_id() {
                    app.command_tx.send(AppCommand::AddAlbumToQueue(id))?;
                }
            }
            MediaBrowserPane::Track => {
                if let Some(id) = app.media_browser.selected_track_id() {
                    app.command_tx.send(AppCommand::AddTrackToQueue(id))?;
                }
            }
        },

        (KeyCode::Char('c'), _) => {
            // Clear the queue and current index, but if the audio is playing
            // keep it playing
            app.queue.clear();
            app.current_queue_idx = None;
        }

        _ => {}
    }

    Ok(())
}
