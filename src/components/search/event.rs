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

//! Event routing for the search view.
//!
//! This module implements the application event processor for the search
//! interface, delegating keyboard input to the underlying track table when the
//! view is active.

use std::sync::mpsc::Sender;

use anyhow::Result;
use crossterm::event::Event;

use crate::{actions::events::{AppEvent, AppEventProcessor}, components::SearchView};

impl AppEventProcessor for SearchView {
    fn process_event(&mut self, event: Event, event_tx: &Sender<AppEvent>) -> Result<()> {
        if self.is_active {
            return self.track_table.process_event(event, event_tx)
        }

        Ok(())
    }
}
