// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

pub mod sdcard;

use crate::Result;

#[trait_variant::make(Storage: Send)]
pub trait BlockDevice {
    /// Reads a 512 byte block from the device.
    async fn read(&mut self, i: u32, buf: &mut [u8; 512]) -> Result<()>;

    /// Writes a 512 byte block to the device.
    async fn write(&mut self, i: u32, buf: &[u8; 512]) -> Result<()>;
}
