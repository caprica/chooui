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

//! User interface rendering logic.
//!
//! This module handles the translation of the [`App`] state into visual
//! widgets using the `ratatui` framework. It is responsible for layout
//! management, widget styling, and terminal frame composition.
//!
//! # Rendering Pipeline
//!
//! The primary entry point is the [`draw`] function, which is called on every
//! terminal tick or state change to provide a reactive user interface.

mod browser;
mod commander;
mod icons;
mod player;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Padding, Paragraph},
};

use crate::{
    App,
    render::{commander::draw_commander, player::draw_player},
    theme::Theme,
};

pub(crate) trait Render {
    fn draw(&mut self, f: &mut Frame, area: Rect, theme: &Theme);
}

/// Renders the user interface to the terminal frame.
///
/// This function calculates the layout constraints and populates the frame
/// with widgets based on the current state of the [`App`].
///
/// It handles:
///
/// * **Layout**: Partitioning the screen into headers, lists, and status bars.
/// * **State Mapping**: Converting application data (like the current track list)
///   into interactive widgets.
/// * **Styling**: Applying colors and borders defined in the application theme.
///
/// # Arguments
///
/// * `f` - The current terminal frame used for drawing.
/// * `app` - A mutable reference to the application state, allowing the UI
///   to reflect changes and update internal view state (like list scroll
///   positions).
pub(crate) fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Outer layout: header, main, footer
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(7),
            Constraint::Length(1),
        ])
        .split(area);

    // Main layout: sidebar, content
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(0), Constraint::Min(0)])
        .split(outer[0]);

    match app.main_view {
        crate::MainView::Playlist => app.playlist_view.draw(f, main[1], &app.queue, &app.theme),
        crate::MainView::Search => app.search_view.draw(f, main[1], &app.search, &app.theme),
        crate::MainView::Favourites => app.favourites_view.draw(f, main[1], &app.theme),
        crate::MainView::Browse => browser::draw_browser(f, main[1], &mut app.media_browser),
    };

    draw_player(f, outer[1], app);

    draw_commander(f, outer[2], app);
}
