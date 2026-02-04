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

//! Visual styling and color configuration for the TUI.
//!
//! This module defines the application's color palette and provides utilities
//! for converting colors between Ratatui's internal representation and external
//! formats (such as hexadecimal strings) used for terminal emulator styling.

use ratatui::style::Color;

#[derive(Clone, Copy)]
pub(crate) struct Theme {
    pub(crate) background_colour: Color,
    pub(crate) accent_colour: Color,
    pub(crate) border_colour: Color,
    pub(crate) gauge_track_colour: Color,

    pub(crate) table_time_fg: Color,
    pub(crate) table_artist_fg: Color,
    pub(crate) table_year_fg: Color,
    pub(crate) table_album_fg: Color,
    pub(crate) table_track_number_fg: Color,
    pub(crate) table_track_fg: Color,
}

impl Default for Theme {
    // Returns the standard application theme.
    fn default() -> Self {
        Self::default_theme()
    }
}

impl Theme {
    // Constructs the default theme.
    pub(crate) const fn default_theme() -> Self {
        Self {
            background_colour: Color::Rgb(40, 20, 50),
            accent_colour: Color::Rgb(250, 189, 47),
            border_colour: Color::Rgb(102, 102, 102),
            gauge_track_colour: Color::Rgb(50, 30, 60),

            table_time_fg: Color::Rgb(162, 161, 166),
            table_artist_fg: Color::Rgb(255, 215, 0),
            table_year_fg: Color::Rgb(162, 161, 166),
            table_album_fg: Color::Rgb(179, 157, 219),
            table_track_number_fg: Color::Rgb(162, 161, 166),
            table_track_fg: Color::Rgb(255, 255, 255),
        }
    }

    /// Converts a [`ratatui::style::Color`] into a CSS-style hexadecimal
    /// string.
    ///
    /// This is primarily used to set the terminal emulator's background color
    /// via escape sequences.
    ///
    /// # Arguments
    ///
    /// * `colour` - The Ratatui color to convert. Must be an `Rgb` variant.
    ///
    /// # Panics
    ///
    /// Panics if the provided color is not a [`Color::Rgb`] variant.
    pub(crate) fn to_hex(colour: Color) -> String {
        match colour {
            Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
            _ => panic!("Unexpected non-RGB colour"),
        }
    }
}
