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
            let offset = 466 + (i * 16);

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

#[derive(Debug)]
pub struct Fat32BiosParameterBlock {
    /// BPB_BytsPerSec: Count of bytes per sector.
    /// This value may take on only the following values: 512, 1024, 2048 or 4096.
    pub bytes_per_sector: u16,

    /// BPB_SecPerClus: Number of sectors per allocation unit.
    /// The legal values are 1, 2, 4, 8, 16, 32, 64, and 128.
    pub sectors_per_cluster: u8,

    /// BPB_RsvdSecCnt: Number of reserved sectors in the Reserved region of the volume starting at the first sector of the volume.
    pub reserved_sectors: u16,

    /// BPB_NumFATs: The count of FAT data structures on the volume.
    /// This field should always contain the value 2 for any FAT volume of any type.
    pub num_fats: u8,

    /// BPB_TotSec32: Total count of sectors on the volume.
    /// This count includes the count of all sectors in all four regions of the volume.
    pub total_sectors32: u32,

    /// BPB_FATSz32: Count of sectors occupied by ONE FAT.
    pub fat_size32: u32,

    /// BPB_RootClus: The cluster number of the first cluster of the root directory, usually 2.
    pub root_cluster: u32,
}

impl Fat32BiosParameterBlock {
    pub(crate) async fn parse(buf: [u8; 512]) -> Self {
        Self {
            bytes_per_sector: u16::from_le_bytes([buf[11], buf[12]]),
            sectors_per_cluster: buf[13],
            reserved_sectors: u16::from_le_bytes([buf[14], buf[15]]),
            num_fats: buf[16],
            total_sectors32: u32::from_le_bytes([buf[32], buf[33], buf[34], buf[35]]),
            fat_size32: u32::from_le_bytes([buf[36], buf[37], buf[38], buf[39]]),
            root_cluster: u32::from_le_bytes([buf[44], buf[45], buf[46], buf[47]]),
        }
    }
}
