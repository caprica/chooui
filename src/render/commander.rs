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

//! Render the command-line interface.
//!
//! This module provides renders the visual representation of the
//! command-line, the current text, the cursor and so on.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::Paragraph,
};

use crate::App;

pub(crate) fn draw_commander(f: &mut Frame, area: Rect, app: &App) {
    let commander = &app.commander;

    let container = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1)])
        .horizontal_margin(1)
        .split(area);

    f.render_widget(
        Paragraph::new(commander.input.value()).style(
            Style::default()
                .fg(app.theme.commander_colour)
                .bg(app.theme.gauge_track_colour),
        ),
        container[0],
    );

    if commander.active() {
        let cursor_x = container[0].x + commander.input.cursor() as u16;
        let cursor_y = container[0].y;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}
