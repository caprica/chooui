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

//! Command-line input logic and state management.
//!
//! This module implements the logic for the for a command-line processing
//! component, handling a text input component, and dispatching a corresponding
//! application command event when typing is finished and a command is
//! submitted.

use std::sync::mpsc::Sender;

use anyhow::Result;
use crossterm::event::{Event, KeyCode};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{MainView, RepeatMode, events::AppEvent, model::SearchQuery, tasks::AppTask};

pub(crate) struct Commander {
    active: bool,
    pub(crate) input: Input,
}

impl Commander {
    pub(crate) fn new() -> Self {
        Self {
            active: false,
            input: Input::default(),
        }
    }

    pub(crate) fn active(&self) -> bool {
        self.active
    }

    pub(crate) fn handle_event(
        &mut self,
        event: Event,
        task_tx: &mut Sender<AppTask>,
        event_tx: &mut Sender<AppEvent>,
    ) -> bool {
        if self.active {
            match event {
                Event::Key(key_event) => {
                    match key_event.code {
                        KeyCode::Esc => {
                            self.active = false;
                            true
                        }

                        KeyCode::Enter => {
                            let buffer = self.input.value().trim();
                            if buffer.len() > 0 {
                                // We need to validate the command as well, and report errors
                                let _ = self.run_command(buffer, task_tx, event_tx);
                                self.input.reset();
                                self.active = false;
                            }

                            true
                        }

                        _ => {
                            // Delegate all key events to the managed input component.
                            if let Event::Key(_) = event {
                                self.input.handle_event(&event);
                            }

                            true
                        }
                    }
                }

                _ => false,
            }
        } else {
            match event {
                Event::Key(key_event) => match key_event.code {
                    KeyCode::Char(':') => {
                        self.active = true;
                        true
                    }

                    _ => false,
                },

                _ => false,
            }
        }
    }

    fn run_command(
        &self,
        buffer: &str,
        task_tx: &mut Sender<AppTask>,
        event_tx: &mut Sender<AppEvent>,
    ) -> Result<()> {
        let parts: Vec<&str> = buffer.split_whitespace().collect();

        match parts.as_slice() {
            ["q"] => event_tx.send(AppEvent::ExitApplication)?,

            ["scan"] => task_tx.send(AppTask::ScanCatalog)?,

            ["asp"] => event_tx.send(AppEvent::AddSelectionToPlaylist)?,

            // The idea is this these will find based on the currently selected item
            ["far"] => {}
            ["fal"] => {}
            ["ftr"] => {}

            ["far", artist_parts @ ..] => {
                if !artist_parts.is_empty() {
                    let name = artist_parts.join(" ");
                    let query = SearchQuery::for_artist(name);
                    task_tx.send(AppTask::Search(query))?
                } else {
                    // error
                }
            }
            ["fal", album_parts @ ..] => {
                if !album_parts.is_empty() {
                    let name = album_parts.join(" ");
                    let query = SearchQuery::for_album(name);
                    task_tx.send(AppTask::Search(query))?
                } else {
                    // error
                }
            }
            ["ftr", track_parts @ ..] => {
                if !track_parts.is_empty() {
                    let name = track_parts.join(" ");
                    let query = SearchQuery::for_track(name);
                    task_tx.send(AppTask::Search(query))?
                } else {
                    // error
                }
            }

            ["qar"] => event_tx.send(AppEvent::AddSelectedArtistToQueue)?,
            ["qal"] => event_tx.send(AppEvent::AddSelectedAlbumToQueue)?,
            ["qtr"] => event_tx.send(AppEvent::AddSelectedTrackToQueue)?,

            ["qar", artist_parts @ ..] => {
                if !artist_parts.is_empty() {
                    let artist = artist_parts.join(" ");
                    task_tx.send(AppTask::AddMatchingArtistToQueue(artist))?
                } else {
                    // error
                }
            }
            ["qal", album_parts @ ..] => {
                if !album_parts.is_empty() {
                    let album = album_parts.join(" ");
                    task_tx.send(AppTask::AddMatchingAlbumToQueue(album))?
                } else {
                    // error
                }
            }
            ["qtr", track_parts @ ..] => {
                if !track_parts.is_empty() {
                    let track = track_parts.join(" ");
                    task_tx.send(AppTask::AddMatchingTrackToQueue(track))?
                } else {
                    // error
                }
            }

            ["cq"] => event_tx.send(AppEvent::ClearQueue)?,

            ["md"] => {} // mode default
            ["ms"] => {} // mode shuffle

            ["p"] => {}  // play/pause
            ["pn"] => {} // play next
            ["pp"] => {} // play previous

            ["v", volume] => {} // volume set
            ["vc", delta] => {} // volume change by delta
            ["vu"] => {}        // volume up
            ["vd"] => {}        // volume down
            ["vm"] => {}        // volume mute

            ["sp"] => {} // show player
            ["hp"] => {} // hide player

            ["sq"] => {} // show queue
            ["hq"] => {} // hide queue

            ["pq"] => event_tx.send(AppEvent::PlayPlaylist)?,

            // maybe vq vb vs vf vc etc?
            ["1"] => event_tx.send(AppEvent::SetMainView(MainView::Playlist))?,
            ["2"] => event_tx.send(AppEvent::SetMainView(MainView::Browse))?,
            ["3"] => event_tx.send(AppEvent::SetMainView(MainView::Search))?,
            ["4"] => event_tx.send(AppEvent::SetMainView(MainView::Favourites))?,
            ["5"] => event_tx.send(AppEvent::SetMainView(MainView::Catalog))?,

            ["repeat", mode_str] => {
                let mode = match mode_str.to_lowercase().as_str() {
                    "all" => Some(RepeatMode::RepeatAll),
                    "one" => Some(RepeatMode::RepeatOne),
                    "none" => Some(RepeatMode::NoRepeat),
                    _ => {
                        println!("Invalid mode: use 'all', 'one', or 'none'");
                        None
                    }
                };

                if let Some(m) = mode {
                    // set repeat mode via event/command?
                }
            }

            [] => {} // empty (no command)

            [cmd, ..] => {} // unknown command (and params)
        }

        Ok(())
    }
}
