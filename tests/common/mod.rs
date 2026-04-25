// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

use std::{fs::File, io::Read};

use polyfs::{Error, block_device::BlockDevice};

pub struct RamDisk {
    blocks: Vec<[u8; 512]>,
}

#[allow(dead_code)]
impl RamDisk {
    pub fn new(num_blocks: u32) -> Self {
        Self {
            blocks: vec![[0u8; 512]; num_blocks as usize],
        }
    }

    pub fn from_file(path: &str) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let remainder = data.len() % 512;
        if remainder != 0 {
            data.extend(std::iter::repeat_n(0, 512 - remainder));
        }

        let num_blocks = data.len() / 512;
        let mut blocks = Vec::with_capacity(num_blocks);

        for chunk in data.chunks_exact(512) {
            let mut block = [0u8; 512];
            block.copy_from_slice(chunk);
            blocks.push(block);
        }

        Ok(Self { blocks })
    }
}

impl BlockDevice for RamDisk {
    async fn read(&mut self, i: u32, buf: &mut [u8; 512]) -> polyfs::Result<()> {
        let block = self.blocks.get(i as usize).ok_or(Error::OutOfBounds)?;
        buf.copy_from_slice(block);
        Ok(())
    }
    async fn write(&mut self, i: u32, buf: &[u8; 512]) -> polyfs::Result<()> {
        let block = self.blocks.get_mut(i as usize).ok_or(Error::OutOfBounds)?;
        block.copy_from_slice(buf);
        Ok(())
    }
}
