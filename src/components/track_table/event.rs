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

use crossterm::event::{Event, KeyCode, KeyModifiers};

use crate::components::{TrackTable, TrackTableAction};

impl TrackTable {
    pub(crate) fn process_event(&mut self, event: &Event) -> Option<TrackTableAction> {
        // Internal events
        match event {
            Event::Key(key_event) => match (key_event.code, key_event.modifiers) {
                (KeyCode::Char('j'), _) | (KeyCode::Down, _) => self.goto_next(),
                (KeyCode::Char('k'), _) | (KeyCode::Up, _) => self.goto_previous(),
                (KeyCode::Char('g'), _) => self.goto_first(),
                (KeyCode::Char('G'), _) => self.goto_last(),

                (KeyCode::Char('H'), _) => self.goto_high(),
                (KeyCode::Char('M'), _) => self.goto_middle(),
                (KeyCode::Char('L'), _) => self.goto_low(),

                (KeyCode::Char('f'), KeyModifiers::CONTROL) => self.goto_page_forward(),
                (KeyCode::Char('b'), KeyModifiers::CONTROL) => self.goto_page_back(),
                (KeyCode::Char('d'), KeyModifiers::CONTROL) => self.goto_half_page_forward(),
                (KeyCode::Char('u'), KeyModifiers::CONTROL) => self.goto_half_page_back(),

                (KeyCode::Char('a'), KeyModifiers::CONTROL) => self.select_all(),
                (KeyCode::Char('t'), KeyModifiers::CONTROL) => self.select_inverse(),
                (KeyCode::Char('l'), KeyModifiers::CONTROL) => self.select_none(),

                (KeyCode::Char(' '), _) => {
                    self.toggle_select_current();
                    self.goto_next();
                }

                (KeyCode::Backspace, _) => {
                    self.toggle_select_current();
                    self.goto_previous();
                }

                _ => {}
            },

            _ => {}
        }

        // External events that result in a table action
        let action = match event {
            Event::Key(key_event) => match (key_event.code, key_event.modifiers) {
                (KeyCode::Enter, _) => Some(&self.selection)
                    .filter(|s| !s.is_empty())
                    .map(|s| TrackTableAction::CommitSelection(s.clone())),

                _ => None,
            },

            _ => None,
        };

        action
    }
}
