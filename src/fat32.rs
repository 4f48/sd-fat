// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

use crate::{
    Error, Result,
    block_device::BlockDevice,
    structs::{Fat32BiosParameterBlock, MasterBootRecord},
};

#[allow(async_fn_in_trait)]
pub trait Fat32: BlockDevice + Sized {
    async fn mount(&self) -> Result<()>;
}

impl<T: BlockDevice> Fat32 for T {
    async fn mount(&self) -> Result<()> {
        let mut sector_0 = [0u8; 512];
        self.read(0, &mut sector_0).await?;

        let mbr = MasterBootRecord::parse(&sector_0)?;
        let fat32_start = mbr
            .partitions
            .iter()
            .find(|p| p.is_fat32())
            .map(|p| p.start)
            .ok_or(Error::NoPartition)?;

        let mut buf = [0u8; 512];
        self.read(fat32_start, &mut buf).await?;
        let bpb = Fat32BiosParameterBlock::parse(buf);
        todo!();
    }
}
