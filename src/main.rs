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

//! # Music Player TUI.
//!
//! A terminal-based music library manager and player.
//!
//! This application coordinates a TUI frontend built with `ratatui` and a
//! background processing layer.
//!
//! It uses an event-driven architecture where:
//!
//! * The **Main Thread** manages the terminal lifecycle and UI rendering.
//! * **Background Workers** handle database queries and long-running tasks via
//!   asynchronous command processing.
//! * **Event Loops** capture user input and system ticks to drive the UI
//!   state.
//!
//! ## Architecture
//!
//! The application follows a strict setup-run-teardown pattern to ensure the
//! terminal state is preserved even in the event of a crash. Communication
//! between the UI and background workers is handled via `std::sync::mpsc`
//! channels.

mod actions;
mod browser;
mod commander;
mod components;
mod config;
mod db;
mod model;
mod player;
mod render;
mod theme;
mod util;

use anyhow::{Context, Result};
use crossterm::{
    event::{self},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    io::{self},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use crate::{
    actions::{
        commands::AppCommand,
        events::{AppEvent, process_events},
    },
    browser::MediaBrowser,
    commander::Commander,
    components::{PlaylistView, SearchView},
    config::AppConfig,
    model::{TrackInfo, queue::Queue, search::Search},
    player::{AudioPlayer, PlayerState},
    theme::Theme,
};

#[derive(Debug, PartialEq)]
enum MainView {
    Playlist,
    Search,
    Browse,
}

/// Application state.
struct App {
    pub config: AppConfig,

    pub theme: Theme,
    pub main_view: MainView,

    pub event_tx: Sender<AppEvent>,
    pub event_rx: Receiver<AppEvent>,

    pub command_tx: Sender<AppCommand>,

    pub audio_player: AudioPlayer,

    pub queue: Queue,
    pub search: Search,

    pub search_view: SearchView,
    pub playlist_view: PlaylistView,

    pub commander: Commander,
    pub media_browser: MediaBrowser,

    pub player_state: PlayerState,
    pub now_playing: Option<TrackInfo>,
    pub player_track_name: Option<String>,
    pub player_duration: Option<u64>,
    pub player_time: Option<u64>,
    pub player_position: Option<f64>,
    pub volume: Option<u32>,
}

impl App {
    /// Create a new instance of application state.
    pub fn new(config: AppConfig, database_tx: Sender<AppCommand>) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::channel();

        let audio_player_event_tx = event_tx.clone();

        let queue = Queue::new();
        let playlist_tracks = queue.tracks();

        let search = Search::new();
        let search_tracks = search.tracks();

        Ok(Self {
            config,
            theme: Theme::default(),
            main_view: MainView::Playlist,
            event_tx,
            event_rx,
            command_tx: database_tx,
            audio_player: AudioPlayer::new(audio_player_event_tx)?,
            queue,
            search,
            search_view: SearchView::new(search_tracks),
            playlist_view: PlaylistView::new(playlist_tracks),
            commander: Commander::new(),
            media_browser: MediaBrowser::new(),
            player_state: PlayerState::Stopped,
            now_playing: None,
            player_track_name: None,
            player_duration: None,
            player_time: None,
            player_position: None,
            volume: None,
        })
    }
}

/// The entry point of the application.
///
/// Sets up the communication channels, initializes the application state,
/// manages the terminal lifecycle, and returns an error if any part of the
/// execution fails.
fn main() -> Result<()> {
    let config = config::load_config();

    let (database_tx, database_rx) = mpsc::channel();

    let mut app = App::new(config, database_tx).context("Failed to initalise application")?;

    let mut terminal = setup_terminal(&app)?;
    let res = run(&mut terminal, &mut app, database_rx);
    restore_terminal(&mut terminal);

    res.context("Application error occurred")
}

/// Prepares the terminal for the TUI application.
///
/// This function performs the following side effects:
/// * Sets the terminal background color based on the provided theme.
/// * Enables raw mode to capture all keyboard input.
/// * Switches the terminal to the alternate screen buffer.
///
/// # Errors
///
/// Returns an error if raw mode cannot be enabled or if the alternate screen
/// cannot be entered.
fn setup_terminal(app: &App) -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    // Set the background of the entire terminal window, without this we'd get
    // a thin black outline
    util::term::set_terminal_bg(&theme::Theme::to_hex(app.theme.background_colour));

    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend).context("Failed to create terminal")?;

    Ok(terminal)
}

/// Restores the terminal to its original state.
///
/// This reverses the changes made by [`setup_terminal`], including disabling
/// raw mode, leaving the alternate screen, and resetting the background color.
/// It also ensures the cursor is made visible again.
///
/// This function is designed to be "best-effort" and does not return a result,
/// as it is typically called during cleanup or panic handling.
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    util::term::reset_terminal_bg();
    terminal.show_cursor().ok();
}

/// Starts the application's background workers and enters the main event loop.
///
/// This function spawns several long-running background threads:
/// * A command worker to process asynchronous [`AppCommand`]s.
/// * An input thread to poll for system keyboard events.
/// * A tick thread to trigger periodic UI refreshes.
///
/// After spawning the workers, it hands control to [`process_events`] to
/// manage the UI and state updates.
///
/// # Errors
///
/// Returns an error if the event processing loop encounters an unrecoverable
/// application error.
fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    command_rx: Receiver<AppCommand>,
) -> Result<()> {
    // Spawn a background worker to process application commands asynchronously.
    let command_event_tx = app.event_tx.clone();
    actions::commands::spawn_command_worker(&app.config, command_rx, command_event_tx);

    // Spawn a thread to translate raw key events to application events.
    let tx_keys = app.event_tx.clone();
    thread::spawn(move || {
        loop {
            if let Ok(event::Event::Key(key)) = event::read() {
                tx_keys.send(AppEvent::Key(key)).ok();
            }
        }
    });

    // Spawn a thread to send a periodic tick application event, this is
    // effectively the minimum "frame rate" for rendering the TUI application.
    let tx_tick = app.event_tx.clone();
    thread::spawn(move || {
        loop {
            let _ = tx_tick.send(AppEvent::Tick);
            thread::sleep(Duration::from_millis(250));
        }
    });

    // Initial trigger to populate the media browser with data from the catalog
    app.command_tx.send(AppCommand::GetBrowserArtists).unwrap();

    // Application event loop, process events until the user quits
    process_events(terminal, app)
}
