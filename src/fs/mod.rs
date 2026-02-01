pub mod fat32;

use crate::{Result, block_device::BlockDevice};
use heapless::{String, Vec};

#[allow(async_fn_in_trait)]
pub trait FileSystem<BD: BlockDevice>: Sized {
    type Directory<'a>: Dir
    where
        Self: 'a;

    async fn mount(device: BD) -> Result<Self>;
    fn open_dir_at(&mut self, sector: u32) -> Self::Directory<'_>;
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
