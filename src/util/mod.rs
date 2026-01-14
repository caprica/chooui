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

//! Common utilities and helper functions.
//!
//! This module provides shared logic used across the application to handle
//! data transformation and environment interactions.
//!
//! # Sub-modules
//!
//! * [`format`]: Logic for converting raw data into human-readable strings for
//!   the UI.
//! * [`terminal`]: Low-level utilities for interacting with the terminal
//!   emulator, such as color control and raw mode management.

pub(crate) mod format;
pub(crate) mod term;
