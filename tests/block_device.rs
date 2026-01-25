// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

mod common;

#[cfg(test)]
mod tests {
    use super::*;
    use common::RamDisk;
    use sd_fat::block_device::BlockDevice;

    #[tokio::test]
    async fn ramdisk_read_write() {
        let mut disk = RamDisk::new(1024);

        let i = fastrand::u32(..1024);
        let block: [u8; 512] = std::array::from_fn(|_| fastrand::u8(..));
        disk.write(i, &block).await.unwrap();

        let mut buf = [0u8; 512];
        disk.read(i, &mut buf).await.unwrap();

        assert_eq!(block, buf)
    }
}
