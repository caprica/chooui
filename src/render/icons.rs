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

//! Unicode and Emoji symbols for the TUI.
//!
//! This module contains standardized icons used across the interface to
//! represent media controls and system status. These are selected for
//! compatibility with most modern terminal emulators and fonts.

// Standard Media Controls (Unicode)
pub(crate) const ICON_PLAY: &str = "\u{25B6}";
pub(crate) const ICON_PAUSE: &str = "\u{23F8}";
pub(crate) const ICON_STOP: &str = "\u{23F9}";
pub(crate) const ICON_NEXT: &str = "\u{23ED}";
pub(crate) const ICON_PREV: &str = "\u{23EE}";

// Text-style variants (using Variation Selector-15 [\u{FE0E}]), this forces
// terminals to render the icons as monochrome text rather than colorful
// emojis, ensuring they respect the TUI's color styling.
pub(crate) const ICON_FF: &str = "\u{23E9}\u{FE0E}";
pub(crate) const ICON_RW: &str = "\u{23EA}\u{FE0E}";

// Volume State Icons (Unicode Speaker Symbols)
pub(crate) const ICON_VOLUME_HIGH: &str = "\u{23F2}";
pub(crate) const ICON_VOLUME_MEDIUM: &str = "\u{1F509}";
pub(crate) const ICON_VOLUME_LOW: &str = "\u{1F508}";
pub(crate) const ICON_MUTED: &str = "\u{1F507}";

pub(crate) const THUMB_UP: &str = "\u{1F44D}";
pub(crate) const THUMB_DOWN: &str = "\u{1F44E}";
pub(crate) const FAVOURITE: &str = "\u{2764}";
