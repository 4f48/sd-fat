// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

use sd_fat::{Error, block_device::BlockDevice};

pub struct RamDisk {
    blocks: Vec<[u8; 512]>,
}

impl RamDisk {
    pub fn new(num_blocks: u32) -> Self {
        Self {
            blocks: vec![[0u8; 512]; num_blocks as usize],
        }
    }
}

impl BlockDevice for RamDisk {
    async fn read(&self, i: u32, buf: &mut [u8; 512]) -> sd_fat::Result<()> {
        let block = self.blocks.get(i as usize).ok_or(Error::OutOfBounds)?;
        buf.copy_from_slice(block);
        Ok(())
    }
    async fn write(&mut self, i: u32, buf: &[u8; 512]) -> sd_fat::Result<()> {
        let block = self.blocks.get_mut(i as usize).ok_or(Error::OutOfBounds)?;
        block.copy_from_slice(buf);
        Ok(())
    }
}
