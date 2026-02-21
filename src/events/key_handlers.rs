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

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent};

use crate::{App, MainView, browser::MediaBrowserPane, events::AppEvent, tasks::AppTask};

const FINE_VOLUME_DELTA: i32 = 1;
const VOLUME_DELTA: i32 = 5;

const FINE_SEEK_DELTA: i32 = 5;
const SEEK_DELTA: i32 = 20;

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
pub(super) fn process_key_event(app: &mut App, key: KeyEvent) -> Result<()> {
    let event = Event::Key(key);
    let handled = app
        .commander
        .handle_event(event.clone(), &mut app.task_tx, &mut app.event_tx);
    if handled {
        return Ok(());
    }

    if app.playlist_view.is_active {
        let event = Event::Key(key);
        app.playlist_view
            .process_event(event, &app.task_tx, &app.event_tx)?;
    }

    if app.search_view.is_active {
        let event = Event::Key(key);
        app.search_view
            .process_event(event, &app.task_tx, &app.event_tx)?;
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
            .event_tx
            .send(AppEvent::SetMainView(MainView::Playlist))?,
        (KeyCode::Char('2'), _) => app.event_tx.send(AppEvent::SetMainView(MainView::Search))?,
        (KeyCode::Char('3'), _) => app
            .event_tx
            .send(AppEvent::SetMainView(MainView::Favourites))?,
        (KeyCode::Char('4'), _) => app.event_tx.send(AppEvent::SetMainView(MainView::Browse))?,
        (KeyCode::Char('5'), _) => app
            .event_tx
            .send(AppEvent::SetMainView(MainView::Catalog))?,

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
                    app.task_tx.send(AppTask::AddArtistToQueue(id))?;
                }
            }
            MediaBrowserPane::Album => {
                if let Some(id) = app.media_browser.selected_album_id() {
                    app.task_tx.send(AppTask::AddAlbumToQueue(id))?;
                }
            }
            MediaBrowserPane::Track => {
                if let Some(id) = app.media_browser.selected_track_id() {
                    app.task_tx.send(AppTask::AddTrackToQueue(id))?;
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
