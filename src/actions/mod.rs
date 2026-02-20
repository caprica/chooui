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

//! Application logic, event handling, and command dispatching.
//!
//! This module acts as the central hub for the "Controller" logic of the
//! application. It organizes how various inputs are translated into internal
//! state changes.
//!
//! # Organization
//!
//! * [`events`]: Defines the raw input types (keyboard, media player, tick
//!   events).
//! * [`commands`]: Contains high-level application commands (add to queue and
//!   so on).
//!
//! All public members of sub-modules are re-exported at this level for
//! convenient access.

pub(crate) mod events;
