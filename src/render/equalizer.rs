use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Borders},
};

/// Represents the state of your 18-band equalizer
struct AppState {
    // Each index corresponds to mpv bands 00-17
    // Values stored as f64 from -24.0 to 12.0
    equalizer_gains: [f64; 18],
}

fn render_equalizer(f: &mut Frame, area: Rect, state: &AppState) {
    let bars: Vec<Bar> = state
        .equalizer_gains
        .iter()
        .enumerate()
        .map(|(i, &gain)| {
            // Convert f64 (-24 to 12) to a relative u64 (0 to 36)
            let ui_value = (gain + 24.0) as u64;

            // Explicitly create the label as a Line
            let label_text = format!("B{:02}", i);

            Bar::default()
                .value(ui_value)
                .label(Line::from(label_text)) // Explicitly use Line::from
                .style(Style::default().fg(if gain > 0.0 {
                    Color::Green
                } else {
                    Color::Cyan
                }))
        })
        .collect();

    let chart = BarChart::default()
        .block(
            Block::default()
                .title(" Equalizer (18-Band) ")
                .borders(Borders::ALL),
        )
        .data(BarGroup::default().bars(&bars))
        .bar_width(3)
        .bar_gap(1)
        .max(36);

    f.render_widget(chart, area);
}
