// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

mod common;

#[cfg(test)]
mod tests {
    use super::*;
    use common::RamDisk;
    use sd_fat::fs::Dir;
    use sd_fat::fs::FileSystem;
    use sd_fat::fs::fat32::Fat32;

    #[tokio::test]
    async fn mount_fs() {
        let disk = RamDisk::from_file("tests/disk.img").unwrap();
        let mut fs = Fat32::mount(disk).await.unwrap();
        let mut root = fs.open_dir("/").await.unwrap();
        println!("{:?}", root.list().await.unwrap());
        println!("{:?}", root.find("BLOCK_~1.RS").await.unwrap());

        let mut docs = fs.open_dir("/FOLDER/DOCS").await.unwrap();
        println!("{:?}", docs.list().await.unwrap());
    }
}
