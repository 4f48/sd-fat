// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

pub mod fat32;

use crate::{Error, Result, block_device::BlockDevice};
use embedded_io_async::Read;
use heapless::{String, Vec};

/// The result of successfully opening a path. Either a directory or a file.
pub enum Entry<D, F> {
    Directory(D),
    File(F),
}

impl<D, F> Entry<D, F> {
    /// Returns the directory, otherwise an error.
    pub fn dir(self) -> Result<D> {
        match self {
            Entry::Directory(d) => Ok(d),
            Entry::File(_) => Err(Error::NotDirectory),
        }
    }

    /// Returns the file, otherwise an error.
    pub fn file(self) -> Result<F> {
        match self {
            Entry::File(f) => Ok(f),
            Entry::Directory(_) => Err(Error::NotFile),
        }
    }

    pub fn is_dir(&self) -> bool {
        match self {
            Entry::Directory(_) => true,
            Entry::File(_) => false,
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait FileSystem<BD: BlockDevice>: Sized {
    type Directory<'a>: Dir
    where
        Self: 'a;

    type File<'b>: Read + File
    where
        Self: 'b;

    /// Mount the drive by parsing the partition table and
    /// extracting essential information for executing file system operations.
    async fn mount(device: BD) -> Result<Self>;

    /// Open file or directory found at the given path.
    async fn open(&mut self, path: &str) -> Result<Entry<Self::Directory<'_>, Self::File<'_>>>;

    /// Open a directory found at the specified sector on disk.
    fn open_dir_at(&mut self, sector: u32) -> Self::Directory<'_>;

    /// Open a directory based on its path.
    #[deprecated]
    async fn open_dir(&mut self, path: &str) -> Result<Self::Directory<'_>>;

    fn open_file_at(&mut self, cluster: u32, size: u32) -> Self::File<'_>;
}

#[allow(async_fn_in_trait)]
pub trait Dir: Sized {
    type Entry: DirEntry;

    /// List all entries in the directory, supports up to 64 entries.
    async fn list(&mut self) -> Result<Vec<Self::Entry, 64>>;

    /// Find a specific entry in the directory by name.
    async fn find(&mut self, name: &str) -> Result<Self::Entry>;
}

pub trait File {
    fn size(&self) -> u32;
}

pub trait DirEntry: Sized {
    fn name(&self) -> &String<12>;
    fn is_dir(&self) -> bool;
    fn size(&self) -> u32;
    fn cluster(&self) -> u32;
}
