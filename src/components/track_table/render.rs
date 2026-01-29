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

//! UI rendering logic for the track table.
//!
//! This module handles the visual representation of track data, including
//! column layout, selection highlighting, and theme application using the
//! Ratatui widget system.

use ratatui::{Frame, layout::{Alignment, Constraint, Rect}, style::{Color, Style}, text::Line, widgets::{Block, Cell, Row, Table}};

use crate::{components::TrackTable, theme::Theme, views::Render};

impl Render for TrackTable<'_> {
    fn draw(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        self.draw_table(f, area, theme);
    }
}

impl TrackTable<'_> {
    fn draw_table(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        let rows = self.tracks.iter().map(|item| {
            let selected = self.selection.contains(&item.track_id);
            let selection_indicator = if selected {
                Line::from("+").style(Style::default().fg(Color::Black).bg(theme.accent_colour))
            } else {
                Line::from("")
            };

            let duration: u64 = item.duration.try_into().unwrap_or(0);
            let time = crate::util::format::format_time(duration, false);

            let track_number = format!("{:02}", item.track_number);

            Row::new(vec![
                Cell::from(selection_indicator),
                Cell::from(Line::from(time).style(Style::default().fg(theme.table_time_fg)).alignment(Alignment::Right)),
                Cell::from(""),
                Cell::from(Line::from(item.artist_name.as_str()).style(Style::default().fg(theme.table_artist_fg))),
                Cell::from(Line::from(item.album_title.as_str()).style(Style::default().fg(theme.table_album_fg))),
                Cell::from(Line::from(track_number).style(Style::default().fg(theme.table_track_number_fg)).alignment(Alignment::Right)),
                Cell::from(Line::from(item.track_title.as_str()).style(Style::default().fg(theme.table_track_fg))),
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(1),
                Constraint::Length(6),
                Constraint::Length(1),
                Constraint::Percentage(20),
                Constraint::Percentage(25),
                Constraint::Length(5),
                Constraint::Percentage(55),
            ],
        )
        .header(
            Row::new(vec![
                Cell::from(""),
                Cell::from(Line::from("Time").alignment(Alignment::Right)),
                Cell::from(""),
                Cell::from("Artist"),
                Cell::from("Album"),
                Cell::from(Line::from("Track").alignment(Alignment::Right)),
                Cell::from("Title"),
            ])
            .style(ratatui::style::Style::default().bold().fg(theme.accent_colour))
            .bottom_margin(1),
        )
        .row_highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
        .block(Block::default());

        let state = &mut self.table_state;
        f.render_stateful_widget(table, area, state);
    }
}
