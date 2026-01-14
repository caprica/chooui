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

/// Formats a duration in seconds into a human-readable `MM:SS` string.
///
/// This is used primarily for displaying track positions and total durations
/// in the player interface.
///
/// # Arguments
///
/// * `total_seconds` - The duration to format, represented as a 64-bit integer.
///
/// # Examples
///
/// ```
/// assert_eq!(format_time(65), "01:05");
/// assert_eq!(format_time(3600), "60:00");
/// ```
pub(crate) fn format_time(total_seconds: u64) -> String {
    let mins = total_seconds / 60;
    let secs = total_seconds % 60;
    format!("{:02}:{:02}", mins, secs)
}
