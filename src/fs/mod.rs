// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

pub mod fat32;

use crate::{Result, block_device::BlockDevice};
use heapless::{String, Vec};

#[allow(async_fn_in_trait)]
pub trait FileSystem<BD: BlockDevice>: Sized {
    type Directory<'a>: Dir
    where
        Self: 'a;

    /// Mount the drive by parsing the partition table and
    /// extracting essential information for executing file system operations.
    async fn mount(device: BD) -> Result<Self>;

    /// Open a directory found at the specified sector on disk.
    fn open_dir_at(&mut self, sector: u32) -> Self::Directory<'_>;

    /// Open a directory based on its path.
    async fn open_dir(&mut self, path: &str) -> Result<Self::Directory<'_>>;
}

#[allow(async_fn_in_trait)]
pub trait Dir: Sized {
    type Entry: DirEntry;

    /// List all entries in the directory, supports up to 64 entries.
    async fn list(&mut self) -> Result<Vec<Self::Entry, 64>>;

    /// Find a specific entry in the directory by name.
    async fn find(&mut self, name: &str) -> Result<Self::Entry>;
}

pub trait DirEntry: Sized {
    fn name(&self) -> &String<12>;
    fn is_dir(&self) -> bool;
    fn size(&self) -> u32;
}
