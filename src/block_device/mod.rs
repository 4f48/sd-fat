// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

pub mod sdcard;

use crate::Result;

#[trait_variant::make(Storage: Send)]
pub trait BlockDevice {
    async fn read(&mut self, i: u32, buf: &mut [u8; 512]) -> Result<()>;
    async fn write(&mut self, i: u32, buf: &[u8; 512]) -> Result<()>;
}
