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

use crate::{App, MainView, browser::MediaBrowserPane, events::AppEvent, tasks::AppTask};

const FINE_VOLUME_DELTA: i32 = 1;
const VOLUME_DELTA: i32 = 5;

const FINE_SEEK_DELTA: i32 = 5;
const SEEK_DELTA: i32 = 20;

pub(super) fn exit_application(app: &mut App) -> Result<()> {
    app.event_tx.send(AppEvent::ExitApplication)?;
    Ok(())
}

pub(super) fn set_view(app: &mut App, view: MainView) -> Result<()> {
    app.event_tx.send(AppEvent::SetMainView(view))?;
    Ok(())
}

pub(super) fn move_selection(app: &mut App, direction: i8) -> Result<()> {
    match app.media_browser.active_pane {
        MediaBrowserPane::Artist => {
            if direction > 0 {
                app.media_browser.next_artist()
            } else {
                app.media_browser.previous_artist()
            };
            if let Some(id) = app.media_browser.selected_artist_id() {
                app.event_tx.send(AppEvent::ArtistSelectionChanged(id))?;
            }
        }
        MediaBrowserPane::Album => {
            if direction > 0 {
                app.media_browser.next_album()
            } else {
                app.media_browser.previous_album()
            };
            if let Some(id) = app.media_browser.selected_album_id() {
                app.event_tx.send(AppEvent::AlbumSelectionChanged(id))?;
            }
        }
        MediaBrowserPane::Track => {
            if direction > 0 {
                app.media_browser.next_track()
            } else {
                app.media_browser.previous_track()
            };
            if let Some(id) = app.media_browser.selected_track_id() {
                app.event_tx.send(AppEvent::TrackSelectionChanged(id))?;
            }
        }
    }
    Ok(())
}

pub(super) fn seek_fine(app: &mut App, forward: bool) -> Result<()> {
    let delta = if forward {
        FINE_SEEK_DELTA
    } else {
        -FINE_SEEK_DELTA
    };
    app.audio_player.seek(delta)
}

pub(super) fn seek_coarse(app: &mut App, forward: bool) -> Result<()> {
    let delta = if forward { SEEK_DELTA } else { -SEEK_DELTA };
    app.audio_player.seek(delta)
}

pub(super) fn adjust_volume_fine(app: &mut App, increase: bool) -> Result<()> {
    let delta = if increase {
        FINE_VOLUME_DELTA
    } else {
        -FINE_VOLUME_DELTA
    };
    app.audio_player.adjust_volume(delta)
}

pub(super) fn adjust_volume_coarse(app: &mut App, increase: bool) -> Result<()> {
    let delta = if increase {
        VOLUME_DELTA
    } else {
        -VOLUME_DELTA
    };
    app.audio_player.adjust_volume(delta)
}

pub(super) fn toggle_playback(app: &mut App) -> Result<()> {
    app.audio_player.toggle_pause()
}

pub(super) fn stop_playback(app: &mut App) -> Result<()> {
    app.audio_player.stop()
}

pub(super) fn toggle_mute(app: &mut App) -> Result<()> {
    app.audio_player.toggle_mute()
}

pub(super) fn add_selected_to_queue(app: &mut App) -> Result<()> {
    let task = match app.media_browser.active_pane {
        MediaBrowserPane::Artist => app
            .media_browser
            .selected_artist_id()
            .map(AppTask::AddArtistToQueue),
        MediaBrowserPane::Album => app
            .media_browser
            .selected_album_id()
            .map(AppTask::AddAlbumToQueue),
        MediaBrowserPane::Track => app
            .media_browser
            .selected_track_id()
            .map(AppTask::AddTrackToQueue),
    };
    if let Some(t) = task {
        app.task_tx.send(t)?;
    }
    Ok(())
}

pub(super) fn clear_queue(app: &mut App) {
    app.queue.clear();
    app.current_queue_idx = None;
}
