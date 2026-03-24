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

//! UI rendering logic for the equalizer view.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin},
    prelude::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Padding, Paragraph},
};

use crate::{components::EqualizerView, model::equalizer::Equalizer, theme::Theme};

const FREQUENCIES: [&str; 18] = [
    "20", "40", "63", "100", "160", "250", "400", "500", "630", "800", "1.2k", "2.5k", "5k", "8k",
    "10k", "12k", "15k", "20k",
];

impl EqualizerView {
    pub(crate) fn draw(&mut self, f: &mut Frame, area: Rect, equalizer: &Equalizer, theme: &Theme) {
        // 1. Root Vertical Layout
        let root_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .split(area);

        // Header
        f.render_widget(
            Paragraph::new("Equalizer Settings").block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .padding(Padding::horizontal(1)),
            ),
            root_chunks[0],
        );

        // 2. Centering Logic (Updated Width)
        // (19 bars * 3) + (18 gaps * 1) + 2 (Block borders) + 2 (Extra gap for Preamp) = 79
        let v_center = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(14),
                Constraint::Fill(1),
            ])
            .split(root_chunks[1]);

        let h_center = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(79),
                Constraint::Fill(1),
            ])
            .split(v_center[1]);

        let content_area = h_center[1];

        // 3. Horizontal Split: [Preamp (3)] [Gap (2)] [18-Bands (72)]
        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(5), // Preamp + Border space
                Constraint::Length(2), // The Gap
                Constraint::Min(0),    // The 18 Bands
            ])
            .split(content_area.inner(Margin {
                horizontal: 1,
                vertical: 1,
            }));

        let amps = equalizer.amps.lock().unwrap();

        // --- Render Preamp ---
        // mpv property: "equalizer-preamp"
        let preamp_val = (amps.preamp + 24.0) as u64;
        let preamp_bar = BarChart::default()
            .block(Block::default().title("P"))
            .data(
                BarGroup::default().bars(&[Bar::default()
                    .value(preamp_val)
                    .label(Line::from("PRE"))
                    .style(Style::default().fg(Color::Magenta))]),
            )
            .bar_width(3)
            .max(36);
        f.render_widget(preamp_bar, main_layout[0]);

        // --- Render 18 Bands ---
        let bars: Vec<Bar> = amps
            .gains
            .iter()
            .enumerate()
            .map(|(i, &gain)| {
                let ui_value = (gain + 24.0) as u64;
                Bar::default()
                    .value(ui_value)
                    .label(Line::from(FREQUENCIES[i]))
                    .style(Style::default().fg(if gain > 0.0 {
                        Color::Green
                    } else {
                        Color::Cyan
                    }))
            })
            .collect();

        let chart = BarChart::default()
            .data(BarGroup::default().bars(&bars))
            .bar_width(3)
            .bar_gap(1)
            .max(36);

        // Wrap the 18 bands in a block to give it the "EQ" title
        f.render_widget(
            chart.block(Block::default().title(" Bands (Hz) ")),
            main_layout[2],
        );

        // Outer container for the whole EQ section
        f.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .title(" 18-Band Audio Processor "),
            content_area,
        );
    }
}
