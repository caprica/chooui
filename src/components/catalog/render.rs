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

//! UI rendering logic for the catalog view.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Cell, Padding, Paragraph, Row, Table},
};

use crate::{
    components::CatalogView,
    model::catalog::{Catalog, CatalogStatus},
    theme::Theme,
};

impl CatalogView {
    pub(crate) fn draw(&mut self, f: &mut Frame, area: Rect, catalog: &Catalog, _theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .split(area);

        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .padding(Padding::horizontal(1));

        let header_content = match catalog.status {
            CatalogStatus::Idle => "Media Catalog (Idle)".to_string(),
            CatalogStatus::Scanning => {
                let total_files: usize = catalog.directory_status.iter().map(|d| d.count).sum();
                format!("Media Catalog (Scanning...) | Total Files: {}", total_files)
            }
            CatalogStatus::Finished => "Media Catalog (Scan Complete)".to_string(),
        };

        let header = Paragraph::new(header_content).block(header_block);
        f.render_widget(header, chunks[0]);

        let header_cells = vec![
            Cell::from("Status"),
            Cell::from(Line::from("Count").alignment(Alignment::Right)),
            Cell::from("Directory"),
        ];

        let header_row = Row::new(header_cells)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .height(1)
            .bottom_margin(1);

        let rows = catalog.directory_status.iter().map(|dir| {
            let (status_text, status_style) = match dir.status {
                CatalogStatus::Idle => ("Pending", Style::default().fg(Color::DarkGray)),
                CatalogStatus::Scanning => (
                    "Scanning",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::ITALIC),
                ),
                CatalogStatus::Finished => ("Done", Style::default().fg(Color::Green)),
            };

            Row::new(vec![
                Cell::from(status_text).style(status_style),
                Cell::from(Line::from(dir.count.to_string()).alignment(Alignment::Right)),
                Cell::from(dir.name.as_str()),
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Min(20),
            ],
        )
        .header(header_row)
        .block(Block::default().padding(Padding::horizontal(1)))
        .column_spacing(2);

        f.render_widget(table, chunks[1]);
    }
}
