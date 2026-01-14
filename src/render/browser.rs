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

//! Render the media browser interface.
//!
//! This module provides renders the visual representation of the music
//! catalog, organised via artist, artist albums, and album tracks.

use ratatui::{
    Frame, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, widgets::{Block, Borders, List, ListItem, ListState}
};

use crate::browser::{MediaBrowser, MediaBrowserPane};

/// Renders the media browser widget including artist, album, and track info.
pub(crate) fn draw_browser(f: &mut Frame, area: Rect, browser: &mut MediaBrowser) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(30),
            Constraint::Percentage(45),
        ])
        .split(area);

    let artist_items: Vec<ListItem> = browser.artists
        .iter()
        .map(|a| ListItem::new(a.name.as_str()))
        .collect();

    render_list(f, chunks[0], " Artists ", artist_items,
        &mut browser.artists_state, browser.active_pane == MediaBrowserPane::Artist);

    let album_items: Vec<ListItem> = browser.albums
        .iter()
        .map(|a| ListItem::new(a.title.as_str()))
        .collect();

    render_list(f, chunks[1], " Albums ", album_items,
        &mut browser.albums_state, browser.active_pane == MediaBrowserPane::Album);

    let width = browser.tracks.len().to_string().len().max(2);
    let track_items: Vec<ListItem> = browser.tracks
        .iter()
        .map(|t| ListItem::new(format!("{:0width$} {}", t.track_number, t.title)))
        .collect();

    render_list(f, chunks[2], " Tracks ", track_items,
        &mut browser.tracks_state, browser.active_pane == MediaBrowserPane::Track);
}

fn render_list(
    f: &mut Frame,
    area: Rect,
    title: &str,
    items: Vec<ListItem>,
    state: &mut ListState,
    is_active: bool,
) {
    let style = if is_active {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(style))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, state);
}
