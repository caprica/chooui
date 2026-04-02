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

//! Input handling and event processing for the audio equalizer.
//!
//! This module maps raw terminal keyboard events to equalizer navigation,
//! selection logic, and delegate notifications.

use anyhow::Result;
use std::sync::mpsc::Sender;

use crossterm::event::{Event, KeyCode};

use crate::{events::AppEvent, model::equalizer::Equalizer, tasks::AppTask};

use super::{AMP_STEP, EqualizerSelection, EqualizerView, MAX_AMP, MIN_AMP};

impl EqualizerView {
    pub(crate) fn process_event(
        &mut self,
        event: &Event,
        _task_tx: &Sender<AppTask>,
        event_tx: &Sender<AppEvent>,
        equalizer: &Equalizer,
    ) -> Result<()> {
        if !self.is_active {
            return Ok(());
        }

        match event {
            Event::Key(key_event) => match (key_event.code, key_event.modifiers) {
                // Navigation: left/right to select bands
                (KeyCode::Char('j'), _) | (KeyCode::Right, _) => self.goto_next(),
                (KeyCode::Char('k'), _) | (KeyCode::Left, _) => self.goto_previous(),
                (KeyCode::Char('g'), _) => self.goto_first(),
                (KeyCode::Char('G'), _) => self.goto_last(),

                // Adjustment: up/down to change amp values
                (KeyCode::Up, _) | (KeyCode::Char('H'), _) => {
                    self.adjust_selected_amp(AMP_STEP, event_tx, equalizer);
                }
                (KeyCode::Down, _) | (KeyCode::Char('L'), _) => {
                    self.adjust_selected_amp(-AMP_STEP, event_tx, equalizer);
                }

                // Page up/down for larger adjustments
                (KeyCode::PageUp, _) => {
                    self.adjust_selected_amp(AMP_STEP * 2.0, event_tx, equalizer);
                }
                (KeyCode::PageDown, _) => {
                    self.adjust_selected_amp(-AMP_STEP * 2.0, event_tx, equalizer);
                }

                // Home/End to set to min/max
                (KeyCode::Home, _) => {
                    self.set_selected_amp(MIN_AMP, event_tx, equalizer);
                }
                (KeyCode::End, _) => {
                    self.set_selected_amp(MAX_AMP, event_tx, equalizer);
                }

                // Reset selected band to 0
                (KeyCode::Char('0'), _) => {
                    self.set_selected_amp(0.0, event_tx, equalizer);
                }

                _ => {}
            },

            _ => {}
        }

        Ok(())
    }

    // FIXME do we really need to send events here? in any case each amp change requires two key-presses for some reason

    /// Adjust the currently selected amp by the given delta.
    fn adjust_selected_amp(
        &mut self,
        delta: f64,
        event_tx: &Sender<AppEvent>,
        equalizer: &Equalizer,
    ) {
        let (index, current_value) = self.get_current_amp_values(equalizer);
        let new_value = (current_value + delta).clamp(MIN_AMP, MAX_AMP);
        if let Err(e) = event_tx.send(AppEvent::UpdateEqualizerAmp(index, new_value)) {
            eprintln!("Failed to send UpdateEqualizerAmp event: {:?}", e);
        }
    }

    /// Set the currently selected amp to a specific value.
    fn set_selected_amp(&mut self, value: f64, event_tx: &Sender<AppEvent>, equalizer: &Equalizer) {
        let (index, _) = self.get_current_amp_values(equalizer);
        if let Err(e) = event_tx.send(AppEvent::UpdateEqualizerAmp(index, value)) {
            eprintln!("Failed to send UpdateEqualizerAmp event: {:?}", e);
        }
    }

    /// Get the index and current value of the selected amp.
    /// Returns (0, preamp_value) for preamp, or (band_index+1, gain_value) for bands.
    fn get_current_amp_values(&self, equalizer: &Equalizer) -> (usize, f64) {
        let amps = equalizer.amps.lock().unwrap();
        match self.selected {
            EqualizerSelection::Preamp => (0, amps.preamp),
            EqualizerSelection::Band(i) => (i + 1, amps.gains[i]),
        }
    }
}
