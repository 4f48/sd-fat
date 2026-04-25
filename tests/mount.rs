// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

mod common;

#[cfg(test)]
mod tests {
    use super::*;
    use common::RamDisk;
    use sd_fat::fs::Dir;
    use sd_fat::fs::File;
    use sd_fat::fs::FileSystem;
    use sd_fat::fs::fat32::Fat32;

    #[tokio::test]
    async fn mount_fs() {
        let disk = RamDisk::from_file("tests/disk.img").unwrap();
        let mut fs = Fat32::mount(disk).await.unwrap();
        let mut root = fs.open("/").await.unwrap().dir().unwrap();
        println!("{:?}", root.list().await.unwrap());
        println!("{:?}", root.find("BLOCK_~1.RS").await.unwrap());

        let doc = fs
            .open("/FOLDER/DOCS/DUALIZ~1.PDF")
            .await
            .unwrap()
            .file()
            .unwrap();
        println!("{:?}", doc.size());
    }
}
