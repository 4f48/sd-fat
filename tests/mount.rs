// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

mod common;

#[cfg(test)]
mod tests {
    use super::*;
    use common::RamDisk;
    use polyfs::fs::Dir;
    use polyfs::fs::File;
    use polyfs::fs::FileSystem;
    use polyfs::fs::fat32::Fat32;

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
