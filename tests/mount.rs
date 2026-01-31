// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

mod common;

#[cfg(test)]
mod tests {
    use super::*;
    use common::RamDisk;
    use sd_fat::fs::FileSystem;
    use sd_fat::fs::fat32::Fat32;
    use sd_fat::fs::{Dir, DirEntry};

    #[tokio::test]
    async fn mount_fs() {
        let disk = RamDisk::from_file("tests/disk.img").unwrap();
        let mut fs = Fat32::mount(disk).await.unwrap();
        let list = fs.open_dir(2).list().await.unwrap();
        println!("{:?}", list);
    }
}
