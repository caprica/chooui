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

//! UI rendering logic for the help view.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
};

use crate::{components::HelpView, theme::Theme};

const SECTION_STYLE: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);

impl HelpView {
    pub(crate) fn draw(&mut self, f: &mut Frame, area: Rect, _theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .split(area);

        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .padding(Padding::horizontal(1));

        let header = Paragraph::new("Help").block(header_block);
        f.render_widget(header, chunks[0]);

        let col_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        let left_lines = build_left_column();
        let right_lines = build_right_column();

        let left =
            Paragraph::new(left_lines).block(Block::default().padding(Padding::horizontal(1)));
        let right =
            Paragraph::new(right_lines).block(Block::default().padding(Padding::horizontal(1)));

        f.render_widget(left, col_chunks[0]);
        f.render_widget(right, col_chunks[1]);
    }
}

fn build_left_column() -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = Vec::new();

    section_title(&mut lines, "Views");
    kv(&mut lines, "0", "Help");
    kv(&mut lines, "1", "Playlist");
    kv(&mut lines, "2", "Browse");
    kv(&mut lines, "3", "Search");
    kv(&mut lines, "4", "Favourites");
    kv(&mut lines, "5", "Equalizer");
    kv(&mut lines, "6", "Catalog");
    blank(&mut lines);

    section_title(&mut lines, "Playback");
    kv(&mut lines, "Space", "Toggle play/pause");
    kv(&mut lines, "n", "Next track");
    kv(&mut lines, "p", "Previous track");
    kv(&mut lines, "s", "Stop playback");
    kv(&mut lines, ", / .", "Fine seek backward/forward");
    kv(&mut lines, "< / >", "Coarse seek backward/forward");
    blank(&mut lines);

    section_title(&mut lines, "Volume");
    kv(&mut lines, "- / =", "Fine volume down/up");
    kv(&mut lines, "_ / +", "Coarse volume down/up");
    kv(&mut lines, "m", "Toggle mute");
    blank(&mut lines);

    section_title(&mut lines, "Queue");
    kv(&mut lines, "a", "Add selection to queue");
    kv(&mut lines, "cq", "Clear queue");
    kv(&mut lines, "pq", "Play queue");
    kv(&mut lines, "sq", "Shuffle queue");
    kv(&mut lines, "rq", "Reset queue");
    blank(&mut lines);

    section_title(&mut lines, "General");
    kv(&mut lines, "q", "Quit application");
    kv(&mut lines, ":", "Enter command mode");
    kv(&mut lines, "Esc", "Exit command mode");
    kv(&mut lines, "scan", "Scan catalog");

    lines
}

fn build_right_column() -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = Vec::new();

    section_title(&mut lines, "Repeat");
    kv(&mut lines, "r0", "Repeat: none");
    kv(&mut lines, "r1", "Repeat: one");
    kv(&mut lines, "ra", "Repeat: all");
    blank(&mut lines);

    section_title(&mut lines, "Rating");
    kv(&mut lines, "]", "Like track");
    kv(&mut lines, "[", "Dislike track");
    blank(&mut lines);

    section_title(&mut lines, "Search");
    kv(&mut lines, "far <name>", "Find by artist");
    kv(&mut lines, "fal <name>", "Find by album");
    kv(&mut lines, "ftr <name>", "Find by track");
    blank(&mut lines);

    section_title(&mut lines, "Queue (Commander)");
    kv(&mut lines, "qar", "Add selected artist to queue");
    kv(&mut lines, "qal", "Add selected album to queue");
    kv(&mut lines, "qtr", "Add selected track to queue");
    kv(&mut lines, "qar <name>", "Add matching artist to queue");
    kv(&mut lines, "qal <name>", "Add matching album to queue");
    kv(&mut lines, "qtr <name>", "Add matching track to queue");
    kv(&mut lines, "asp", "Add selection to playlist");
    blank(&mut lines);

    section_title(&mut lines, "Equalizer");
    kv(&mut lines, "re", "Reset equalizer");
    kv(&mut lines, "j / Right", "Next band");
    kv(&mut lines, "k / Left", "Previous band");
    kv(&mut lines, "Up / H", "Increase gain");
    kv(&mut lines, "Down / L", "Decrease gain");
    kv(&mut lines, "g", "First band");
    kv(&mut lines, "G", "Last band");

    lines
}

fn section_title(lines: &mut Vec<Line<'static>>, title: &str) {
    lines.push(Line::from(Span::styled(title.to_string(), SECTION_STYLE)));
}

fn blank(lines: &mut Vec<Line<'static>>) {
    lines.push(Line::from(Span::raw("")));
}

fn kv(lines: &mut Vec<Line<'static>>, key: &str, desc: &str) {
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {:<12}", key),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default().fg(Color::DarkGray)),
        Span::raw(desc.to_string()),
    ]));
}
