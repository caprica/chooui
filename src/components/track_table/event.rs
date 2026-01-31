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

//! Input handling and event processing for the track table.
//!
//! This module maps raw terminal keyboard events to table navigation,
//! selection logic, and delegate notifications.

use anyhow::{Result};
use crossterm::event::{Event, KeyCode, KeyModifiers};

use crate::components::{TrackTable, TrackTableDelegate};

impl TrackTable {
    pub(crate) fn process_event(&mut self, event: Event, delegate: &impl TrackTableDelegate) -> Result<()> {
        match event {
            Event::Key(key_event) => {
                match (key_event.code, key_event.modifiers) {
                    (KeyCode::Char('j'), _) | (KeyCode::Down, _) => self.goto_next(),
                    (KeyCode::Char('k'), _) | (KeyCode::Up, _) => self.goto_previous(),
                    (KeyCode::Char('g'), _) => self.goto_first(),
                    (KeyCode::Char('G'), _) => self.goto_last(),

                    (KeyCode::Char('a'), KeyModifiers::CONTROL) => self.select_all(),
                    (KeyCode::Char('t'), KeyModifiers::CONTROL) => self.select_inverse(),
                    (KeyCode::Char('u'), KeyModifiers::CONTROL) => self.select_none(),

                    (KeyCode::Char(' '), _) => {
                        self.toggle_select_current();
                        self.goto_next();
                    }

                    (KeyCode::Backspace, _) => {
                        self.toggle_select_current();
                        self.goto_previous();
                    }

                    (KeyCode::Enter, _) => delegate.on_activate_selection(),

                    _ => {}
                }
            }
            _ => {}
        }

        Ok(())
    }
}
