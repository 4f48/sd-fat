pub mod fat32;

use crate::{Result, block_device::BlockDevice};
use heapless::{String, Vec};

#[allow(async_fn_in_trait)]
pub trait FileSystem<BD: BlockDevice>: Sized {
    type Directory<'a>: Dir
    where
        Self: 'a;

    async fn mount(device: BD) -> Result<Self>;
    fn open_dir(&mut self, sector: u32) -> Self::Directory<'_>;
}

#[allow(async_fn_in_trait)]
pub trait Dir: Sized {
    type Entry: DirEntry;

    fn info(&self);
    async fn list(&mut self) -> Result<Vec<Self::Entry, 10>>;
}

pub trait DirEntry: Sized {
    fn name(&self) -> &String<12>;
    fn is_dir(&self) -> bool;
}
