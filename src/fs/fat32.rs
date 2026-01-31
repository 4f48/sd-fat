// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

use heapless::{String, Vec};

use crate::{
    Error, Result,
    block_device::BlockDevice,
    error::BadClusterVariant,
    fs::{Dir, DirEntry, FileSystem},
    part::MasterBootRecord,
};

struct BiosParameterBlock {
    /// BPB_BytsPerSec: Count of bytes per sector.
    /// This value may take on only the following values: 512, 1024, 2048 or 4096.
    bytes_per_sector: u16,

    /// BPB_SecPerClus: Number of sectors per allocation unit.
    /// The legal values are 1, 2, 4, 8, 16, 32, 64, and 128.
    sectors_per_cluster: u8,

    /// BPB_RsvdSecCnt: Number of reserved sectors in the Reserved region of the volume starting at the first sector of the volume.
    reserved_sectors: u16,

    /// BPB_NumFATs: The count of FAT data structures on the volume.
    /// This field should always contain the value 2 for any FAT volume of any type.
    num_fats: u8,

    /// BPB_TotSec32: Total count of sectors on the volume.
    /// This count includes the count of all sectors in all four regions of the volume.
    total_sectors32: u32,

    /// BPB_FATSz32: Count of sectors occupied by ONE FAT.
    fat_size32: u32,

    /// BPB_RootClus: The cluster number of the first cluster of the root directory, usually 2.
    root_cluster: u32,
}

impl BiosParameterBlock {
    pub(crate) fn parse(buf: [u8; 512]) -> Self {
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

pub struct Fat32<D: BlockDevice> {
    device: D,
    sectors_per_cluster: u8,

    /// The cluster where the FAT table starts.
    first_fat_sector: u32,

    /// The cluster where data starts.
    first_data_sector: u32,

    /// The cluster where the root directory lives.
    root_cluster: u32,
}

impl<D: BlockDevice> Fat32<D> {
    fn get_sector(&self, cluster: u32) -> u32 {
        let index = cluster.saturating_sub(2);
        let offset = index * self.sectors_per_cluster as u32;
        self.first_data_sector + offset
    }
    async fn next_cluster(&mut self, cluster: u32) -> Result<Option<u32>> {
        let index = cluster * 4;
        let sector = self.first_fat_sector + (index / 512);
        let byte = (index % 512) as usize;

        let mut buf = [0u8; 512];
        self.device.read(sector, &mut buf).await?;
        let raw = u32::from_le_bytes(buf[byte..byte + 4].try_into().unwrap());

        let value = raw & 0x0FFF_FFFF;
        match value {
            0x0000_0000 => Err(Error::BadCluster(BadClusterVariant::Free)),
            0x0000_0001 => Err(Error::BadCluster(BadClusterVariant::Reserved)),
            0x0FFF_FFF8..0x0FFF_FFFF => Ok(None),
            _ => Ok(Some(value)),
        }
    }
}

impl<BD: BlockDevice> FileSystem<BD> for Fat32<BD> {
    type Directory<'a>
        = Fat32Dir<'a, BD>
    where
        Self: 'a;

    async fn mount(mut device: BD) -> Result<Self> {
        let mut sector_0 = [0u8; 512];
        device.read(0, &mut sector_0).await?;

        let mbr = MasterBootRecord::parse(&sector_0)?;
        let lba_start = mbr
            .partitions
            .iter()
            .find(|p| p.is_fat32())
            .map(|p| p.start)
            .ok_or(Error::NoPartition)?;

        let mut buf = [0u8; 512];
        device.read(lba_start, &mut buf).await?;

        if buf[510] != 0x55 || buf[511] != 0xAA {
            return Err(Error::InvalidPartition);
        }

        let bpb = BiosParameterBlock::parse(buf);

        Ok(Self {
            device,
            sectors_per_cluster: bpb.sectors_per_cluster,
            first_fat_sector: lba_start + bpb.reserved_sectors as u32,
            first_data_sector: lba_start
                + bpb.reserved_sectors as u32
                + (bpb.num_fats as u32 * bpb.fat_size32),
            root_cluster: bpb.root_cluster,
        })
    }
    fn open_dir(&mut self, cluster: u32) -> Self::Directory<'_> {
        Fat32Dir {
            fs: self,
            cluster,
            cursor: cluster,
            offset: 0,
        }
    }
}

pub struct Fat32Dir<'a, BD: BlockDevice> {
    fs: &'a mut Fat32<BD>,
    cluster: u32,
    cursor: u32,
    offset: u32,
}

impl<'a, BD: BlockDevice> Dir for Fat32Dir<'a, BD> {
    type Entry = Fat32DirEntry;

    fn info(&self) {
        todo!();
    }
    async fn list(&mut self) -> Result<Vec<Self::Entry, 10>> {
        self.cursor = self.cluster;

        let mut results = Vec::new();

        let mut buf = [0u8; 512];
        'sectors: loop {
            let start_sector = self.fs.first_data_sector
                + ((self.cursor - 2) * self.fs.sectors_per_cluster as u32);

            for i in 0..self.fs.sectors_per_cluster {
                let sector = start_sector + i as u32;
                self.fs.device.read(sector, &mut buf).await?;

                for chunk in buf.chunks(32) {
                    match chunk[0] {
                        0x00 => break 'sectors, // End of Directory
                        0xE5 => continue,       // Deleted file
                        _ => (),
                    }

                    // TODO: Handle LFNs
                    if chunk[11] == 0x0F {
                        continue;
                    }

                    let mut name_str: String<12> = String::new();
                    for i in 0..8 {
                        let b = chunk[i];
                        if b != 0x20 {
                            name_str.push(b as char).map_err(|_| Error::CapacityError)?;
                        }
                    }
                    if chunk[8] != 0x20 {
                        name_str.push('.').map_err(|_| Error::CapacityError)?;
                    }
                    for i in 8..11 {
                        let b = chunk[i];
                        if b != 0x20 {
                            name_str.push(b as char).map_err(|_| Error::CapacityError)?;
                        }
                    }

                    let cluster_hi = u16::from_le_bytes(chunk[20..22].try_into().unwrap());
                    let cluster_lo = u16::from_le_bytes(chunk[26..28].try_into().unwrap());

                    let entry = Fat32DirEntry {
                        name: name_str,
                        is_dir: (chunk[11] & 0x10) != 0,
                        size: u32::from_le_bytes(chunk[28..32].try_into().unwrap()),
                        cluster: ((cluster_hi as u32) << 16) | (cluster_lo as u32),
                    };
                    if let Err(_) = results.push(entry) {
                        return Ok(results);
                    };
                }

                match self.fs.next_cluster(self.cursor).await? {
                    Some(next) => self.cursor = next,
                    None => break 'sectors,
                }
            }
        }

        Ok(results)
    }
}

#[derive(Debug)]
pub struct Fat32DirEntry {
    name: String<12>,
    is_dir: bool,
    size: u32,
    cluster: u32,
}

impl DirEntry for Fat32DirEntry {
    fn name(&self) -> &String<12> {
        &self.name
    }
    fn is_dir(&self) -> bool {
        self.is_dir
    }
}
