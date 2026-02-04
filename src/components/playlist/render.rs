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

use std::fmt::Write;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    prelude::Rect,
    widgets::{Block, Borders, Padding, Paragraph},
};

use crate::{
    components::PlaylistView, model::queue::Queue, render::Render, theme::Theme,
    util::format::TimeFormat,
};

impl PlaylistView {
    pub(crate) fn draw(&mut self, f: &mut Frame, area: Rect, queue: &Queue, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .split(area);

        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .padding(Padding::horizontal(1));

        let track_count = queue.tracks().lock().unwrap().len();
        let total_duration = queue.total_duration();

        let selected_count = self.track_table.selected_count();

        let mut header_text = format!(
            "Playlist | {} tracks | {}",
            track_count,
            crate::util::format::format_time(
                total_duration.try_into().unwrap_or_default(),
                TimeFormat::Hours
            )
        );

        if selected_count > 0 {
            let _ = write!(header_text, " | {} selected", selected_count);
        }

        let header = Paragraph::new(header_text).block(header_block);

        f.render_widget(header, chunks[0]);
        self.track_table.draw(f, chunks[1], theme);
    }
}
