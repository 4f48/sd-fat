// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

//! # polyfs
//!
//! Extensible asynchronous `no_std` and `no_alloc` driver collection for working with storage devices. It currently works with MBR formatted SD cards, including read-only support for the FAT32 file system. While this doesn't seem like much, this crate was built from the ground for async and extensibility. It includes a bunch of generic code to make adding new device types and file systems less terrible.
//!
//! This crate is generic over [embedded-hal-async](https://crates.io/crates/embedded-hal-async) and [embedded-io-async](https://crates.io/crates/embedded-io-async) and thus should be compatible with a wide range of other crates and (some) legacy code out there.
//!
//! ## Current state of polyfs
//! This crate is still in development and lacks many features. Here's what we have so far:
//!
//! | Feature               | Status                         |
//! |-----------------------|--------------------------------|
//! | SD card via SPI       | ✅                             |
//! | MBR partition table   | 🔬 only first partition        |
//! | FAT32 file system     | 🔬 read-only, no LFN support   |
//! | ExFAT file system     | 🚧 planned                     |
//! | SD card via SDIO      | 🚧 planned                     |
//! | GPT partition table   | 🚧 planned                     |
//!
//! ## License
//! polyfs is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//!
//! polyfs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.

#![no_std]

pub mod block_device;
pub mod error;
pub mod fs;
pub mod part;

pub use error::{Error, Result};
