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
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Padding, Paragraph},
};

use crate::{components::EqualizerView, model::equalizer::Equalizer, theme::Theme};

use super::{EqualizerSelection, MAX_AMP, MIN_AMP};

const FREQ_NUMS: [&str; 18] = [
    "20", "40", "63", "100", "160", "250", "400", "500", "630", "800", "1.2", "2.5", "5", "8",
    "10", "12", "15", "20",
];
const FREQ_UNITS: [&str; 18] = [
    "Hz", "Hz", "Hz", "Hz", "Hz", "Hz", "Hz", "Hz", "Hz", "Hz", "kHz", "kHz", "kHz", "kHz", "kHz",
    "kHz", "kHz", "kHz",
];

impl EqualizerView {
    pub(crate) fn draw(&mut self, f: &mut Frame, area: Rect, equalizer: &Equalizer, theme: &Theme) {
        let root_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .split(area);

        f.render_widget(
            Paragraph::new("Equalizer Settings").block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .padding(Padding::horizontal(1)),
            ),
            root_chunks[0],
        );

        let v_center = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(28),
                Constraint::Fill(1),
            ])
            .split(root_chunks[1]);

        let h_center = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(84),
                Constraint::Fill(1),
            ])
            .split(v_center[1]);

        let content_area = h_center[1];

        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(6),
                Constraint::Length(3),
                Constraint::Length(4),
                Constraint::Min(0),
            ])
            .split(content_area.inner(Margin {
                horizontal: 0,
                vertical: 0,
            }));

        let db_labels = Paragraph::new(vec![
            Line::from(Span::styled(
                format!("{:>3}dB", MAX_AMP as i32),
                Style::default().add_modifier(Modifier::UNDERLINED),
            )),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from("  0dB"),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                format!("{:>3}dB", MIN_AMP as i32),
                Style::default().add_modifier(Modifier::UNDERLINED),
            )),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from("   dB"),
        ])
        .style(Style::default().fg(Color::DarkGray));

        f.render_widget(db_labels, main_layout[0]);

        let amps = equalizer.amps.lock().unwrap();
        let selected = self.selection();

        let charts_y_offset = 1;
        let preamp_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(charts_y_offset),
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(main_layout[1]);

        let preamp_val = (amps.preamp - MIN_AMP) as u64;
        let preamp_selected = matches!(selected, EqualizerSelection::Preamp);
        let preamp_bar = BarChart::default()
            .block(Block::default().style(Style::default().bg(Color::Indexed(234))))
            .data(
                BarGroup::default().bars(&[Bar::default()
                    .value(preamp_val)
                    .text_value("".to_string())
                    .label(Line::from("PRE"))
                    .style(Style::default().fg(if preamp_selected {
                        theme.accent_colour
                    } else {
                        Color::Cyan
                    }))]),
            )
            .bar_width(3)
            .max((MAX_AMP - MIN_AMP) as u64);

        f.render_widget(preamp_bar, preamp_chunks[1]);
        f.render_widget(
            Paragraph::new(format!("{:^3.0}", amps.preamp))
                .style(Style::default().fg(theme.accent_colour)),
            preamp_chunks[4],
        );

        let mut bands_constraints = Vec::new();
        for i in 0..18 {
            bands_constraints.push(Constraint::Length(3));
            if i < 17 {
                bands_constraints.push(Constraint::Length(1));
            }
        }
        let bands_columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(bands_constraints)
            .split(main_layout[3]);

        for i in 0..18 {
            let col_idx = i * 2;
            let gain = amps.gains[i];
            let is_selected = matches!(selected, EqualizerSelection::Band(idx) if idx == i);

            let band_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(charts_y_offset),
                    Constraint::Min(0),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .split(bands_columns[col_idx]);

            let bar_chart = BarChart::default()
                .block(Block::default().style(Style::default().bg(Color::Indexed(234))))
                .data(
                    BarGroup::default().bars(&[Bar::default()
                        .value((gain - MIN_AMP) as u64)
                        .text_value("".to_string())
                        .style(Style::default().fg(if is_selected {
                            theme.accent_colour
                        } else {
                            Color::Cyan
                        }))]),
                )
                .bar_width(3)
                .max((MAX_AMP - MIN_AMP) as u64);

            f.render_widget(bar_chart, band_chunks[1]);

            f.render_widget(
                Paragraph::new(format!("{:^3}", FREQ_NUMS[i])).style(Style::default().fg(
                    if is_selected {
                        theme.accent_colour
                    } else {
                        Color::White
                    },
                )),
                band_chunks[2],
            );

            f.render_widget(
                Paragraph::new(format!("{:^3}", FREQ_UNITS[i]))
                    .style(Style::default().fg(Color::DarkGray)),
                band_chunks[3],
            );

            f.render_widget(
                Paragraph::new(format!("{:^3.0}", gain))
                    .style(Style::default().fg(theme.accent_colour)),
                band_chunks[5],
            );
        }
    }
}
