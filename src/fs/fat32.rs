// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

use embedded_io_async::{ErrorType, Read};
use heapless::{String, Vec};

use crate::{
    Error, Result,
    block_device::BlockDevice,
    error::{self, BadClusterVariant},
    fs::{Dir, DirEntry, File, FileSystem},
    part::MasterBootRecord,
};

#[allow(dead_code)]
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
            0x0FFF_FFF8..=0x0FFF_FFFF => Ok(None),
            _ => Ok(Some(value)),
        }
    }
}

impl<'a, BD: BlockDevice> File for Fat32File<'a, BD> {
    fn size(&self) -> u32 {
        self.size
    }
}

impl<BD: BlockDevice> FileSystem<BD> for Fat32<BD> {
    type Directory<'a>
        = Fat32Dir<'a, BD>
    where
        Self: 'a;

    type File<'b>
        = Fat32File<'b, BD>
    where
        Self: 'b;

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

    fn open_dir_at(&mut self, cluster: u32) -> Self::Directory<'_> {
        Fat32Dir {
            fs: self,
            cluster,
            cursor: cluster,
        }
    }

    async fn open_dir(&mut self, path: &str) -> Result<Self::Directory<'_>> {
        let mut dir = self.open_dir_at(self.root_cluster);

        let path = path.trim_start_matches('/');

        if path.is_empty() {
            return Ok(dir);
        }

        for segment in path.split('/') {
            let entry = dir.find(segment).await?;

            if !entry.is_dir() {
                return Err(Error::NotFound);
            }

            dir.cluster = entry.cluster;
        }

        Ok(dir)
    }

    fn open_file_at(&mut self, cluster: u32, size: u32) -> Self::File<'_> {
        Fat32File {
            fs: self,
            first: cluster,
            cluster,
            cursor: 0,
            size,
        }
    }

    async fn open(
        &mut self,
        path: &str,
    ) -> Result<super::Entry<Self::Directory<'_>, Self::File<'_>>> {
        let path = path.trim_start_matches('/');
        if path.is_empty() {
            let root_dir = self.open_dir_at(self.root_cluster);
            return Ok(super::Entry::Directory(root_dir));
        }

        let (segments, last) = match path.rfind('/') {
            Some(i) => (&path[..i], &path[i + 1..]),
            None => ("", path),
        };

        let (cluster, size, is_dir) = {
            let mut dir = self.open_dir_at(self.root_cluster);

            for segment in segments.split('/').filter(|s| !s.is_empty()) {
                let entry = dir.find(segment).await?;
                if !entry.is_dir() {
                    return Err(Error::NotDirectory);
                }
                dir.cluster = entry.cluster;
            }

            let entry = dir.find(last).await?;
            (entry.cluster(), entry.size(), entry.is_dir())
        };

        Ok(if is_dir {
            let dir = self.open_dir_at(cluster);
            super::Entry::Directory(dir)
        } else {
            let file = self.open_file_at(cluster, size);
            super::Entry::File(file)
        })
    }
}

pub struct Fat32Dir<'a, BD: BlockDevice> {
    fs: &'a mut Fat32<BD>,
    cluster: u32,
    cursor: u32,
}

impl<'a, BD: BlockDevice> Dir for Fat32Dir<'a, BD> {
    type Entry = Fat32DirEntry;

    async fn list(&mut self) -> Result<Vec<Self::Entry, 64>> {
        self.cursor = self.cluster;

        let mut results = Vec::new();

        let mut buf = [0u8; 512];
        'sectors: loop {
            let start_sector = self.fs.get_sector(self.cursor);

            for i in 0..self.fs.sectors_per_cluster {
                let sector = start_sector + i as u32;
                self.fs.device.read(sector, &mut buf).await?;

                for chunk in buf.chunks(32) {
                    let entry = match Fat32DirEntry::parse(chunk) {
                        Ok(Some(entry)) => entry,
                        Ok(None) => continue,
                        Err(Error::EndOfChain) => break 'sectors,
                        Err(e) => return Err(e),
                    };
                    if results.push(entry).is_err() {
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

    async fn find(&mut self, name: &str) -> Result<Self::Entry> {
        self.cursor = self.cluster;

        let mut buf = [0u8; 512];
        'sectors: loop {
            let start_sector = self.fs.get_sector(self.cursor);

            for i in 0..self.fs.sectors_per_cluster {
                let sector = start_sector + i as u32;
                self.fs.device.read(sector, &mut buf).await?;

                for chunk in buf.chunks(32) {
                    let entry = match Fat32DirEntry::parse(chunk) {
                        Ok(Some(entry)) => entry,
                        Ok(None) => continue,
                        Err(Error::EndOfChain) => break 'sectors,
                        Err(e) => return Err(e),
                    };
                    if entry.name == name {
                        return Ok(entry);
                    }
                }
            }

            match self.fs.next_cluster(self.cursor).await? {
                Some(next) => self.cursor = next,
                None => break 'sectors,
            }
        }

        Err(Error::NotFound)
    }
}

#[derive(Debug)]
pub struct Fat32DirEntry {
    name: String<12>,
    is_dir: bool,
    size: u32,
    cluster: u32,
}

impl Fat32DirEntry {
    fn parse(chunk: &[u8]) -> Result<Option<Self>> {
        match chunk[0] {
            0x00 => return Err(Error::EndOfChain), // End of Directory
            0xE5 => return Ok(None),               // Deleted file
            _ => (),
        }

        // TODO: Handle LFNs
        if chunk[11] == 0x0F {
            return Ok(None);
        }

        let mut name_str: String<12> = String::new();
        for b in chunk.iter().take(8) {
            if *b != 0x20 {
                name_str
                    .push(*b as char)
                    .map_err(|_| Error::CapacityError)?;
            }
        }
        if chunk[8] != 0x20 {
            name_str.push('.').map_err(|_| Error::CapacityError)?;
        }
        for b in chunk.iter().take(11).skip(8) {
            if *b != 0x20 {
                name_str
                    .push(*b as char)
                    .map_err(|_| Error::CapacityError)?;
            }
        }

        let cluster_hi = u16::from_le_bytes(chunk[20..22].try_into().unwrap());
        let cluster_lo = u16::from_le_bytes(chunk[26..28].try_into().unwrap());

        Ok(Some(Fat32DirEntry {
            name: name_str,
            is_dir: (chunk[11] & 0x10) != 0,
            size: u32::from_le_bytes(chunk[28..32].try_into().unwrap()),
            cluster: ((cluster_hi as u32) << 16) | (cluster_lo as u32),
        }))
    }
}

impl DirEntry for Fat32DirEntry {
    fn name(&self) -> &String<12> {
        &self.name
    }

    fn is_dir(&self) -> bool {
        self.is_dir
    }

    fn size(&self) -> u32 {
        self.size
    }

    fn cluster(&self) -> u32 {
        self.cluster
    }
}

#[allow(dead_code)]
pub struct Fat32File<'a, BD: BlockDevice> {
    fs: &'a mut Fat32<BD>,
    first: u32,
    cluster: u32,
    cursor: u32,
    size: u32,
}

impl<'a, BD: BlockDevice> ErrorType for Fat32File<'a, BD> {
    type Error = error::Error;
}

impl<'a, BD: BlockDevice> Read for Fat32File<'a, BD> {
    async fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, Self::Error> {
        if self.cursor >= self.size || buf.is_empty() {
            return Ok(0);
        }

        let cluster_offset = self.cursor % (512 * self.fs.sectors_per_cluster as u32);
        let sector_offset = (cluster_offset % 512) as usize;

        let mut sector = [0u8; 512];
        self.fs
            .device
            .read(
                self.fs.get_sector(self.cluster) + (cluster_offset / 512),
                &mut sector,
            )
            .await?;

        let n = buf
            .len()
            .min(512 - sector_offset)
            .min((self.size - self.cursor) as usize);
        buf[..n].copy_from_slice(&sector[sector_offset..sector_offset + n]);

        self.cursor += n as u32;

        if self
            .cursor
            .is_multiple_of(512 * self.fs.sectors_per_cluster as u32)
            && self.cursor < self.size
            && let Some(next) = self.fs.next_cluster(self.cluster).await?
        {
            self.cluster = next;
        }

        Ok(n)
    }
}
