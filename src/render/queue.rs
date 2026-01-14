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

//! Render the player queue interface.
//!
//! This module provides renders the visual representation of the current
//! player queue of tracks to be played.

use ratatui::{
    Frame, layout::Rect, style::{Color, Style}, widgets::{Block, Borders, List, ListItem}
};

use crate::{queue::Queue, theme::Theme};

/// Renders the player queue widget including items queued to be played.
pub(crate) fn draw_queue(f: &mut Frame, area: Rect, queue: &Queue, theme: &Theme) {
    let items: Vec<ListItem> = queue.queued()
        .iter()
        .map(|track| {
            ListItem::new(format!("{}", track.track_title))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .title(" Queue ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_colour))
        )
        .highlight_style(Style::default().bg(Color::Cyan))
        .highlight_symbol(">> ");

    f.render_widget(list, area);
}
