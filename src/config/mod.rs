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

//! Application configuration.
//!
//! This module manages the application configuration file.

use serde::{Deserialize, Serialize};

const CONFIG_NAME: &str = "chooui";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub version: u32,
    pub media_dirs: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            media_dirs: vec![],
        }
    }
}

pub fn load_config() -> AppConfig {
    confy::load(CONFIG_NAME, None).unwrap_or_default()
}

pub fn save_config(cfg: &AppConfig) -> Result<(), confy::ConfyError> {
    confy::store(CONFIG_NAME, None, cfg)
}
