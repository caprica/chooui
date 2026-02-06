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

//! Media catalog management.
//!
//! This module provides state for the media catalog scanning process, managing
//! state of the directories and files being scanned.

pub(crate) enum CatalogStatus {
    Idle,
    Scanning,
    Finished,
}

pub(crate) struct DirectoryStatus {
    pub(crate) status: CatalogStatus,
    pub(crate) name: String,
    pub(crate) count: usize,
}

pub(crate) struct Catalog {
    pub(crate) status: CatalogStatus,
    pub(crate) directory_status: Vec<DirectoryStatus>,
    current_directory_index: Option<usize>,
}

impl Catalog {
    pub(crate) fn new() -> Self {
        Self {
            status: CatalogStatus::Idle,
            directory_status: vec![],
            current_directory_index: None,
        }
    }

    pub(crate) fn prepare_scan(&mut self, directories: &Vec<String>) {
        self.status = CatalogStatus::Scanning;

        self.directory_status = directories
            .into_iter()
            .map(|d| DirectoryStatus {
                status: CatalogStatus::Idle,
                name: d.clone(),
                count: 0,
            })
            .collect();
    }

    pub(crate) fn begin_scan_directory(&mut self, directory: &str) {
        self.current_directory_index = self
            .directory_status
            .iter()
            .position(|s| s.name == directory);

        if let Some(idx) = self.current_directory_index {
            self.directory_status[idx].status = CatalogStatus::Scanning;
        }
    }

    pub(crate) fn update_scan_directory(&mut self, count: usize) {
        if let Some(idx) = self.current_directory_index {
            if let Some(status) = self.directory_status.get_mut(idx) {
                status.count = count;
            }
        }
    }

    pub(crate) fn end_scan_directory(&mut self) {
        if let Some(idx) = self.current_directory_index {
            self.directory_status[idx].status = CatalogStatus::Finished;
        }
        self.current_directory_index = None;
    }

    pub(crate) fn finish_scan(&mut self) {
        self.status = CatalogStatus::Finished;
    }
}
