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

//! Terminal environment and styling utilities.
//!
//! This module provides functions to manipulate the terminal emulator's
//! properties, such as background colors, using OSC (Operating System Command)
//! escape sequences.
//!
//! # Compatibility
//!
//! These functions rely on the terminal emulator supporting the specific OSC
//! codes. Most modern terminals (XTerm, iTerm2, Alacritty, Kitty) support these
//! sequences.

use std::io::{self, Write};

/// Sets the terminal background color using an OSC 11 escape sequence.
///
/// This sends a command to the terminal emulator to change its background color
/// to the specified hex string.
///
/// # Arguments
///
/// * `hex_color` - A string slice representing the color (e.g., `"#1e1e1e"`).
///
/// # Note
///
/// This function flushes `stdout` immediately to ensure the change is applied
/// without delay.
pub(crate) fn set_terminal_bg(hex_color: &str) {
    print!("\x1b]11;{}\x07", hex_color);
    io::stdout().flush().unwrap();
}

/// Resets the terminal background to its default color.
///
/// This sends the OSC 111 escape sequence, which instructs the terminal to
/// revert the background color to the user's original configuration.
///
/// # Note
///
/// This is called during application cleanup to ensure the user's terminal
/// state is restored.
pub(crate) fn reset_terminal_bg() {
    print!("\x1b]111\x07");
    io::stdout().flush().unwrap();
}
