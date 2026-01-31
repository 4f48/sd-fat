// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

use crate::{Error, Result};

pub struct MasterBootRecord {
    pub(crate) partitions: [PartitionTableEntry; 4],
}

impl MasterBootRecord {
    pub(crate) fn parse(buf: &[u8; 512]) -> Result<Self> {
        // check valid bootsector signature bytes
        if buf[510] != 0x55 || buf[511] != 0xAA {
            return Err(Error::InvalidPartition);
        }

        let mut partitions = [PartitionTableEntry {
            r#type: 0,
            start: 0,
            num_sectors: 0,
        }; 4];

        for (i, entry) in partitions.iter_mut().enumerate() {
            let offset = 446 + (i * 16);

            entry.r#type = buf[offset + 4];
            entry.start = u32::from_le_bytes(buf[offset + 8..offset + 12].try_into().unwrap());
            entry.num_sectors =
                u32::from_le_bytes(buf[offset + 12..offset + 16].try_into().unwrap());
        }

        Ok(Self { partitions })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PartitionTableEntry {
    r#type: u8,
    pub(crate) start: u32,
    num_sectors: u32,
}

impl PartitionTableEntry {
    pub(crate) fn is_fat32(&self) -> bool {
        // 0x0B: FAT32 with CHS/LBA
        // 0x0C: FAT32 with LBA
        self.r#type == 0x0B || self.r#type == 0x0C
    }
}
