// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

mod common;

#[cfg(test)]
mod tests {
    use super::*;
    use common::RamDisk;
    use embedded_io_async::Read;
    use sd_fat::fs::File;
    use sd_fat::fs::FileSystem;
    use sd_fat::fs::fat32::Fat32;

    #[tokio::test]
    async fn read_file() {
        let disk = RamDisk::from_file("tests/disk.img").unwrap();
        let mut fs = Fat32::mount(disk).await.unwrap();
        let mut file = fs.open("/BLOCK_~1.RS").await.unwrap().file().unwrap();
        let mut buf = vec![0u8; file.size() as usize];
        file.read_exact(&mut buf).await.unwrap();

        let str = String::from_utf8_lossy(&buf);
        println!("{}", str);
    }
}
