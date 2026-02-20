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

//! Audio playback control and state management.
//!
//! This module provides the high-level [`AudioPlayer`] interface used by the
//! UI to control music playback. It manages a background worker thread that
//! interfaces with the underlying audio library (MPV), ensuring that heavy
//! audio operations do not block the main application thread.

mod commands;

use std::sync::mpsc;

use anyhow::Result;

use crate::{events::AppEvent, player::commands::AudioPlayerCommand};

/// Represents the current playback status of the audio engine.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PlayerState {
    Playing,
    Paused,
    Stopped,
}

/// A handle to the audio playback engine.
///
/// This struct acts as a command proxy; it does not perform audio processing
/// itself but instead sends instructions to a background worker thread.
pub(crate) struct AudioPlayer {
    /// Channel for sending commands to the background worker thread.
    command_tx: mpsc::Sender<AudioPlayerCommand>,
}

impl AudioPlayer {
    /// Spawns the audio worker thread and returns a new player handle.
    ///
    /// # Arguments
    ///
    /// * `event_tx` - A channel to send application-level events (like progress
    ///   updates or errors) back to the main event loop.
    pub(crate) fn new(event_tx: mpsc::Sender<AppEvent>) -> Result<Self> {
        let (command_tx, command_rx) = mpsc::channel::<AudioPlayerCommand>();

        commands::spawn_player_worker(command_rx, event_tx);

        Ok(Self { command_tx })
    }

    // Maps internal audio backend flags to a simplified [`PlayerState`].
    fn player_state(is_paused: bool, is_idle: bool) -> PlayerState {
        if is_idle {
            PlayerState::Stopped
        } else if is_paused {
            PlayerState::Paused
        } else {
            PlayerState::Playing
        }
    }

    /// Instructs the worker to load and play a specific audio file.
    ///
    /// # Arguments
    ///
    /// * `filename` - The path to the audio file on disk.
    ///
    pub(crate) fn play_file(&self, filename: &str) -> Result<()> {
        self.command_tx
            .send(AudioPlayerCommand::PlayFile(filename.to_string()))?;
        Ok(())
    }

    /// Toggles the playback state between paused and playing.
    pub(crate) fn toggle_pause(&self) -> Result<()> {
        self.command_tx.send(AudioPlayerCommand::TogglePause)?;
        Ok(())
    }

    /// Stop playback.
    pub(crate) fn stop(&self) -> Result<()> {
        self.command_tx.send(AudioPlayerCommand::Stop)?;
        Ok(())
    }

    /// Adjusts the playback volume relative to the current level.
    ///
    /// # Arguments
    ///
    /// * `delta` - The amount to change the volume (positive or negative).
    pub(crate) fn adjust_volume(&self, delta: i32) -> Result<()> {
        self.command_tx
            .send(AudioPlayerCommand::AdjustVolume(delta))?;
        Ok(())
    }

    /// Toggles the audio output between muted and unmuted.
    pub(crate) fn toggle_mute(&self) -> Result<()> {
        self.command_tx.send(AudioPlayerCommand::ToggleMute)?;
        Ok(())
    }

    /// Adjusts the playback position forward or backwards relative to the
    /// current position.
    ///
    /// # Arguments
    ///
    /// * `delta` - The amount to seek (positive or negative).
    pub(crate) fn seek(&self, delta: i32) -> Result<()> {
        self.command_tx.send(AudioPlayerCommand::Seek(delta))?;
        Ok(())
    }
}
