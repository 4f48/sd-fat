// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

#[derive(Debug)]
pub enum Error {
    OutOfBounds,
    InvalidPartition,
    ConversionError,
    NoPartition,
    BadCluster(BadClusterVariant),
    ClusterFree,
    CapacityError,
}

#[derive(Debug)]
pub enum BadClusterVariant {
    Free,
    Reserved,
}

pub type Result<T> = core::result::Result<T, Error>;
