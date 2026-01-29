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

//! UI rendering logic for the playlist view.
//!
//! This module handles the visual representation of the playlist interface by
//! delegating the drawing process to the underlying track table widget based
//! on the current theme and layout constraints.

use ratatui::{Frame, prelude::Rect};

use crate::{components::PlaylistView, render::Render, theme::Theme};

impl Render for PlaylistView {
    fn draw(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        self.table_state.as_widget().draw(f, area, theme);
    }
}
