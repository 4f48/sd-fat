// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

use core::fmt;

use embedded_io_async::ErrorKind;

#[derive(Debug)]
pub enum Error {
    OutOfBounds,
    InvalidPartition,
    ConversionError,
    NoPartition,
    BadCluster(BadClusterVariant),
    ClusterFree,
    CapacityError,
    NotFound,
    EndOfChain,
    FileDeleted,
    TransferError,
    WriteError,
    CsError,
    NotDirectory,
    NotFile,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::OutOfBounds => write!(f, "out of bounds access"),
            Error::InvalidPartition => write!(f, "invalid partition"),
            Error::ConversionError => write!(f, "conversion error"),
            Error::NoPartition => write!(f, "no partition found"),
            Error::BadCluster(v) => write!(f, "bad cluster: {:?}", v),
            Error::ClusterFree => write!(f, "cluster is free"),
            Error::CapacityError => write!(f, "buffer capacity exceeded"),
            Error::NotFound => write!(f, "not found"),
            Error::EndOfChain => write!(f, "end of cluster chain"),
            Error::FileDeleted => write!(f, "file is deleted"),
            Error::TransferError => write!(f, "transfer error"),
            Error::WriteError => write!(f, "write error"),
            Error::CsError => write!(f, "error setting chip select pin"),
            Error::NotDirectory => write!(f, "not a directory"),
            Error::NotFile => write!(f, "not a file"),
        }
    }
}

impl core::error::Error for Error {}

impl embedded_io_async::Error for Error {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        match self {
            Error::OutOfBounds => ErrorKind::InvalidInput,

            Error::InvalidPartition => ErrorKind::InvalidData,
            Error::ConversionError => ErrorKind::InvalidData,
            Error::BadCluster(_) => ErrorKind::InvalidData,
            Error::ClusterFree => ErrorKind::InvalidData,
            Error::NotDirectory => ErrorKind::InvalidData,
            Error::NotFile => ErrorKind::InvalidData,

            Error::NoPartition => ErrorKind::NotFound,
            Error::NotFound => ErrorKind::NotFound,
            Error::FileDeleted => ErrorKind::NotFound,

            Error::CapacityError => ErrorKind::OutOfMemory,

            Error::EndOfChain => ErrorKind::Other,
            Error::TransferError => ErrorKind::Other,
            Error::CsError => ErrorKind::Other,

            Error::WriteError => ErrorKind::WriteZero,
        }
    }
}

#[derive(Debug)]
pub enum BadClusterVariant {
    Free,
    Reserved,
}

pub type Result<T> = core::result::Result<T, Error>;
