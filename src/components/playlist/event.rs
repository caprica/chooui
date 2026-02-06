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

//! Event routing for the playlist view.
//!
//! This module implements the application event processor for the playlist
//! interface, delegating keyboard input to the underlying track table
//! when the view is active.

use std::sync::mpsc::Sender;

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent};

use crate::{
    actions::{commands::AppCommand, events::AppEvent},
    components::{PlaylistView, TrackTableAction},
    model::Rating,
};

impl PlaylistView {
    pub(crate) fn process_event(
        &mut self,
        event: Event,
        command_tx: &Sender<AppCommand>,
        event_tx: &Sender<AppEvent>,
    ) -> Result<()> {
        if !self.is_active {
            return Ok(());
        }

        if let Some(action) = self.track_table.process_event(&event) {
            return self.handle_table_action(action, event_tx);
        }

        if let Event::Key(key) = event {
            return self.handle_key_event(key, command_tx);
        }

        Ok(())
    }

    fn handle_table_action(
        &self,
        action: TrackTableAction,
        event_tx: &Sender<AppEvent>,
    ) -> Result<()> {
        match action {
            _ => {}
        }

        Ok(())
    }

    fn handle_key_event(&self, key: KeyEvent, command_tx: &Sender<AppCommand>) -> Result<()> {
        match key.code {
            KeyCode::Char(']') => {
                if let Some(track) = self.track_table.clone_current() {
                    command_tx.send(AppCommand::RateTrack(track, Rating::Like))?;
                }
            }

            KeyCode::Char('[') => {
                if let Some(track) = self.track_table.clone_current() {
                    command_tx.send(AppCommand::RateTrack(track, Rating::Dislike))?;
                }
            }

            KeyCode::Char('p') => {
                if let Some(track) = self.track_table.clone_current() {
                    command_tx.send(AppCommand::PlayTrack(track))?;
                }
            }

            _ => {}
        }

        Ok(())
    }
}
