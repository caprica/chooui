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

pub(crate) enum TimeFormat {
    Minutes,
    Hours,
}

/// Formats a duration in seconds into a human-readable `MM:SS` string.
///
/// This is used primarily for displaying track positions and total durations
/// in the player interface.
///
/// # Arguments
///
/// * `total_seconds` - The duration to format, represented as a 64-bit integer.
/// * `style` - The formatting style to apply.
///
/// # Examples
///
/// ```
/// assert_eq!(format_time(65, TimeFormat::Minutes), "1:05");
/// assert_eq!(format_time(3600), "60:00");
/// ```
pub(crate) fn format_time(total_seconds: u64, style: TimeFormat) -> String {
    let hours = total_seconds / 3600;
    let mins = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;

    match style {
        TimeFormat::Minutes => {
            if hours > 0 {
                format!("{hours}:{:02}:{:02}", mins, secs)
            } else {
                format!("{mins}:{:02}", secs)
            }
        }
        TimeFormat::Hours => {
            format!("{hours}:{:02}:{:02}", mins, secs)
        }
    }
}
