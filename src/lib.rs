// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

#![no_std]

pub mod block_device;
pub mod error;
pub mod fat32;
pub mod structs;

pub use error::{Error, Result};
