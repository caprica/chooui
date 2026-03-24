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

//! Input handling and event processing for the audio equalizer.
//!
//! This module maps raw terminal keyboard events to equalizer navigation,
//! selection logic, and delegate notifications.

use anyhow::Result;
use std::sync::mpsc::Sender;

use crossterm::event::{Event, KeyCode, KeyModifiers};

use crate::{components::EqualizerView, events::AppEvent, tasks::AppTask};

impl EqualizerView {
    pub(crate) fn process_event(
        &mut self,
        event: &Event,
        task_tx: &Sender<AppTask>,
        event_tx: &Sender<AppEvent>,
    ) -> Result<()> {
        if !self.is_active {
            return Ok(());
        }
        // Internal events
        match event {
            Event::Key(key_event) => match (key_event.code, key_event.modifiers) {
                // (KeyCode::Char('j'), _) | (KeyCode::Right, _) => self.goto_next(),
                // (KeyCode::Char('k'), _) | (KeyCode::Left, _) => self.goto_previous(),
                // (KeyCode::Char('g'), _) => self.goto_first(),
                // (KeyCode::Char('G'), _) => self.goto_last(),
                _ => {}
            },

            _ => {}
        }

        Ok(())
    }
}
