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

use crate::{MainView, RepeatMode, actions::commands::AppCommand, model::SearchQuery};

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
        command_sender: &mut Sender<AppCommand>,
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
                                let _ = self.run_command(buffer, command_sender);
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

    fn run_command(&self, buffer: &str, command_sender: &mut Sender<AppCommand>) -> Result<()> {
        let parts: Vec<&str> = buffer.split_whitespace().collect();

        match parts.as_slice() {
            ["q"] => command_sender.send(AppCommand::ExitApplication)?,

            ["scan"] => command_sender.send(AppCommand::ScanCatalog)?,

            ["asp"] => command_sender.send(AppCommand::AddSelectionToPlaylist)?,

            // The idea is this these will find based on the currently selected item
            ["far"] => {}
            ["fal"] => {}
            ["ftr"] => {}

            ["far", artist_parts @ ..] => {
                if !artist_parts.is_empty() {
                    let name = artist_parts.join(" ");
                    let query = SearchQuery::for_artist(name);
                    command_sender.send(AppCommand::Search(query))?
                } else {
                    // error
                }
            }
            ["fal", album_parts @ ..] => {
                if !album_parts.is_empty() {
                    let name = album_parts.join(" ");
                    let query = SearchQuery::for_album(name);
                    command_sender.send(AppCommand::Search(query))?
                } else {
                    // error
                }
            }
            ["ftr", track_parts @ ..] => {
                if !track_parts.is_empty() {
                    let name = track_parts.join(" ");
                    let query = SearchQuery::for_track(name);
                    command_sender.send(AppCommand::Search(query))?
                } else {
                    // error
                }
            }

            ["qar"] => command_sender.send(AppCommand::AddSelectedArtistToQueue)?,
            ["qal"] => command_sender.send(AppCommand::AddSelectedAlbumToQueue)?,
            ["qtr"] => command_sender.send(AppCommand::AddSelectedTrackToQueue)?,

            ["qar", artist_parts @ ..] => {
                if !artist_parts.is_empty() {
                    let artist = artist_parts.join(" ");
                    command_sender.send(AppCommand::AddMatchingArtistToQueue(artist))?
                } else {
                    // error
                }
            }
            ["qal", album_parts @ ..] => {
                if !album_parts.is_empty() {
                    let album = album_parts.join(" ");
                    command_sender.send(AppCommand::AddMatchingAlbumToQueue(album))?
                } else {
                    // error
                }
            }
            ["qtr", track_parts @ ..] => {
                if !track_parts.is_empty() {
                    let track = track_parts.join(" ");
                    command_sender.send(AppCommand::AddMatchingTrackToQueue(track))?
                } else {
                    // error
                }
            }

            ["cq"] => {} // clear queue

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

            ["pq"] => command_sender.send(AppCommand::PlayPlaylist)?,

            ["1"] => command_sender.send(AppCommand::SetMainView(MainView::Playlist))?,
            ["2"] => command_sender.send(AppCommand::SetMainView(MainView::Browse))?,
            ["3"] => command_sender.send(AppCommand::SetMainView(MainView::Search))?,
            ["4"] => command_sender.send(AppCommand::SetMainView(MainView::Favourites))?,
            ["5"] => command_sender.send(AppCommand::SetMainView(MainView::Catalog))?,

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
