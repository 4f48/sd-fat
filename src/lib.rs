// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

#![no_std]

pub mod block_device;
pub mod error;
pub mod fs;
pub mod part;

pub use error::{Error, Result};
