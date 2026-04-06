// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

mod common;

#[cfg(test)]
mod tests {
    use super::*;
    use common::RamDisk;
    use embedded_io_async::Read;
    use sd_fat::fs::Dir;
    use sd_fat::fs::DirEntry;
    use sd_fat::fs::FileSystem;
    use sd_fat::fs::fat32::Fat32;

    #[tokio::test]
    async fn read_file() {
        let disk = RamDisk::from_file("tests/disk.img").unwrap();
        let mut fs = Fat32::mount(disk).await.unwrap();
        let mut dir = fs.open_dir("/").await.unwrap();

        let entry = dir.find("BLOCK_~1.RS").await.unwrap();
        println!("{:?}", entry);

        let mut file = fs.open_file_at(entry.cluster(), entry.size());
        let mut buf = vec![0u8; entry.size() as usize];
        file.read_exact(&mut buf).await.unwrap();

        let str = String::from_utf8_lossy(&buf);
        println!("{}", str);
    }
}
