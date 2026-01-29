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

//! Render the music player interface.
//!
//! This module provides renders the visual representation of the current
//! track, playback controls, progress barsm and so on.

use ratatui::{
    Frame, layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style, Stylize}, text::{Line, Span}, widgets::{Block, Borders, Gauge, Padding, Paragraph}
};

use crate::{App, player::PlayerState, render::icons::{ICON_PAUSE, ICON_PLAY, ICON_STOP}, util};

/// Renders the main player widget including track info and controls.
pub(crate) fn draw_player(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::TOP | Borders::BOTTOM)
        .border_style(Style::default().fg(app.theme.border_colour))
        .padding(Padding::horizontal(1));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner_area);

    let info_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(30),
        ])
        .split(chunks[0]);

    if let Some(track_info) = &app.now_playing {
        let icon = match app.player_state {
            PlayerState::Playing => ICON_PLAY,
            PlayerState::Paused => ICON_PAUSE,
            PlayerState::Stopped => ICON_STOP,
        };

        let track_line = Line::from(vec![
            Span::styled(format!(" {} ", icon), Style::default().add_modifier(Modifier::BOLD)).fg(Color::White),
            Span::styled(&track_info.track_title, Style::default().add_modifier(Modifier::BOLD)).fg(app.theme.accent_colour),
            Span::raw(" from "),
            Span::styled(&track_info.album_title, Style::default().add_modifier(Modifier::BOLD)).fg(app.theme.accent_colour),
            Span::raw(" by "),
            Span::styled(&track_info.artist_name, Style::default().add_modifier(Modifier::BOLD)).fg(app.theme.accent_colour),
        ]);
        f.render_widget(Paragraph::new(track_line), info_chunks[0]);

        let duration = app.player_duration.unwrap_or(0);
        let time = app.player_time.unwrap_or(0);
        let remaining = duration.saturating_sub(time);

        let time_line = Line::from(vec![
            Span::styled(util::format::format_time(time), Style::default().add_modifier(Modifier::BOLD)).fg(app.theme.accent_colour),
            Span::styled(" / ", Style::default().add_modifier(Modifier::BOLD)).fg(Color::White),
            Span::styled(util::format::format_time(duration), Style::default().add_modifier(Modifier::BOLD)).fg(app.theme.accent_colour),
            Span::styled(" (-", Style::default().add_modifier(Modifier::BOLD)).fg(Color::White),
            Span::styled(util::format::format_time(remaining), Style::default().add_modifier(Modifier::BOLD)).fg(app.theme.accent_colour),
            Span::styled(")", Style::default().add_modifier(Modifier::BOLD)).fg(Color::White),
        ]);

        let time_p = Paragraph::new(time_line).alignment(Alignment::Right);

        f.render_widget(time_p, info_chunks[1]);
    }

    let control_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(26),
        ])
        .split(chunks[2]);

    let volume = app.volume.unwrap_or(0);
    let vol_ratio = (volume as f64 / 130.0).clamp(0.0, 1.0);

    let volume_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(5),
        ])
        .split(control_chunks[1]);

    let volume_gauge = Gauge::default()
        .gauge_style(Style::default().fg(app.theme.accent_colour).bg(app.theme.gauge_track_colour))
        .ratio(vol_ratio)
        .label("")
        .use_unicode(true);
    f.render_widget(volume_gauge, volume_layout[0]);

    let volume_percent = (vol_ratio * 130.0) as u16;

    let volume_label = Paragraph::new(format!(" {}%", volume_percent))
        .alignment(Alignment::Right)
        .fg(Color::White);
    f.render_widget(volume_label, volume_layout[1]);

    let position = app.player_position.unwrap_or(0.0).clamp(0.0, 1.0);

    let position_gauge = Gauge::default()
        .gauge_style(Style::default()
            .fg(app.theme.accent_colour)
            .bg(app.theme.gauge_track_colour)
        )
        .ratio(position)
        .label("")
        .use_unicode(true);

    f.render_widget(position_gauge, chunks[4]);
}
