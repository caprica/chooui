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

//! MPV-backed audio playback engine and event processing.
//!
//! This module provides the core audio playback logic, leveraging `libmpv` for
//! high-quality audio decoding and playback control. It manages a background
//! worker thread that bridges the gap between the application's command-based
//! interface and the low-level MPV property observation system.
//!
//! # Architecture
//!
//! The engine operates using a dual-channel communication pattern:
//! 1. **Command Channel**: Receives [`AudioPlayerCommand`]s from the UI to
//!    control playback (play, pause, seek, etc.).
//! 2. **Event Channel**: Broadcasts [`AppEvent`]s to notify the UI of state
//!    changes, such as track progress, volume updates, and metadata changes.

// TODO consider splitting this into commands/events like elsewhere?

use anyhow::{Context, Result};
use mpv::Format;
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crate::{
    events::AppEvent,
    player::{AudioPlayer, PlayerState},
};

#[derive(Debug)]
pub(crate) enum AudioPlayerCommand {
    PlayFile(String),
    TogglePause,
    Seek(i32),
    Stop,
    AdjustVolume(i32),
    ToggleMute,
}

/// Spawns the audio worker thread to process playback commands.
///
/// This function takes ownership of the command receiver and the event sender,
/// moving them into a dedicated background thread.
///
/// If the internal worker returns an error, it is caught here and broadcast as
/// a fatal application event.
///
/// # Arguments
///
/// * `command_rx` - The receiving end of the player command channel.
/// * `event_tx` - The channel used to broadcast playback updates and errors.
pub(crate) fn spawn_player_worker(
    command_rx: Receiver<AudioPlayerCommand>,
    event_tx: Sender<AppEvent>,
) {
    let error_tx = event_tx.clone();

    thread::spawn(move || {
        if let Err(e) = audio_player_worker(command_rx, event_tx) {
            let _ = error_tx.send(AppEvent::FatalError(format!("MPV worker failure: {:?}", e)));
        }
    });
}

/// The primary execution loop for the audio player backend.
///
/// This function initializes a local `libmpv` context and enters a multi-loop
/// select pattern to handle incoming commands and outgoing events
/// simultaneously.
///
/// # Arguments
///
/// * `command_rx` - The receiving end of a channel for incoming player commands.
/// * `event_tx` - The sending end of a channel for broadcasting
///    application-level events.
///
/// # Errors
///
/// Returns an error if the MPV context fails to initialize or if the internal
/// command/event loops encounter an unrecoverable failure.
fn audio_player_worker(
    command_rx: Receiver<AudioPlayerCommand>,
    event_tx: Sender<AppEvent>,
) -> Result<()> {
    let mut handler = (|| {
        let mut builder = mpv::MpvHandlerBuilder::new().context("Failed to create MPV builder")?;
        builder
            .set_option("vo", "null")
            .context("Failed to set no video output")?;
        builder.build().context("Failed to build MPV handler")
    })()?;

    handler
        .observe_property::<&str>("media-title", 0)
        .context("Failed to observe media-title")?;
    handler
        .observe_property::<f64>("duration", 0)
        .context("Failed to observe duration")?;
    handler
        .observe_property::<bool>("pause", 0)
        .context("Failed to observe pause")?;
    handler
        .observe_property::<f64>("time-pos", 0)
        .context("Failed to observe time-pos")?;
    handler
        .observe_property::<f64>("volume", 0)
        .context("Failed to observe volume")?;
    handler
        .observe_property::<f64>("idle-active", 0)
        .context("Failed to observe idle-active")?;

    let mut is_paused = false;
    let mut is_idle = true;

    let mut player_state = PlayerState::Stopped;

    loop {
        process_commands(&mut handler, &command_rx)?;
        process_mpv_events(
            &mut handler,
            &mut is_paused,
            &mut is_idle,
            &mut player_state,
            &event_tx,
        )?;
    }
}

/// Drains and executes all pending commands from the application channel.
fn process_commands(
    handler: &mut mpv::MpvHandler,
    command_rx: &mpsc::Receiver<AudioPlayerCommand>,
) -> Result<()> {
    while let Ok(command) = command_rx.try_recv() {
        match command {
            AudioPlayerCommand::PlayFile(filename) => {
                handler
                    .command(&["loadfile", &filename, "replace"])
                    .context(format!("Failed to load file: {}", &filename))?;
                handler.set_property("pause", false)?;
            }
            AudioPlayerCommand::TogglePause => {
                handler.command(&["cycle", "pause"]).unwrap();
            }
            AudioPlayerCommand::Seek(delta) => {
                handler.command(&["seek", &delta.to_string(), "relative"])?;
            }
            AudioPlayerCommand::Stop => {
                handler.command(&["stop"])?;
            }
            AudioPlayerCommand::AdjustVolume(delta) => {
                handler.command(&["add", "volume", &delta.to_string()])?;
            }
            AudioPlayerCommand::ToggleMute => {
                handler.command(&["cycle", "mute"]).unwrap();
            }
        }
    }

    Ok(())
}

/// Polls for MPV events and synchronizes the application state.
///
/// This function waits for up to 50ms for an event from the MPV context.
/// If an event occurs, it updates internal flags and broadcasts any necessary
/// [`AppEvent`]s to the UI.
fn process_mpv_events(
    handler: &mut mpv::MpvHandler,
    is_paused: &mut bool,
    is_idle: &mut bool,
    current_state: &mut PlayerState,
    event_tx: &mpsc::Sender<AppEvent>,
) -> Result<()> {
    if let Some(mpv_event) = handler.wait_event(0.05) {
        let app_event = match mpv_event {
            mpv::Event::PropertyChange { name, change, .. } => match (name, change) {
                ("media-title", Format::Str(title)) => {
                    Some(AppEvent::TitleChanged(title.to_string()))
                }
                ("duration", Format::Double(duration)) => {
                    Some(AppEvent::DurationChanged(duration as u64))
                }
                ("pause", Format::Flag(pause)) => {
                    *is_paused = pause;
                    None
                }
                ("time-pos", Format::Double(seconds)) if seconds >= 0.0 => {
                    Some(AppEvent::TimeChanged(seconds))
                }
                ("volume", Format::Double(volume)) => {
                    Some(AppEvent::VolumeChanged(volume.round() as u32))
                }
                ("idle-active", Format::Flag(idle_active)) => {
                    *is_idle = idle_active;
                    None
                }
                _ => None,
            },
            mpv::Event::EndFile(result) => {
                if let Ok(reason) = result {
                    match reason {
                        mpv::EndFileReason::MPV_END_FILE_REASON_EOF => {
                            Some(AppEvent::TrackFinished)
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        };

        let new_player_state = AudioPlayer::player_state(*is_paused, *is_idle);

        if new_player_state != *current_state {
            *current_state = new_player_state;
            event_tx
                .send(AppEvent::PlayerStateChanged(new_player_state))
                .context("Failed to send player state event")?;
        }

        if let Some(event) = app_event {
            event_tx.send(event).context("Failed to send event")?;
        }
    }

    Ok(())
}
