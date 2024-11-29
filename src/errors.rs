// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*! Definition for low level error class for core methods !*/

use core::fmt;
use std::path::PathBuf;

use crate::deviceinfo::DeviceInfo;

#[derive(Clone, Debug)]
/// Internal error for low-level devicemapper operations
pub enum DmError {
    /// An error returned on failure to create a devicemapper context
    ContextInit(String),

    /// This is a generic error that can be returned when a method
    /// receives an invalid argument. Ideally, the argument should be
    /// invalid in itself, i.e., it should not be made invalid by some
    /// part of the program state or the environment.
    InvalidArgument(String),

    /// An error returned exclusively by DM methods.
    /// This error is initiated in DM::do_ioctl and returned by
    /// numerous wrapper methods.
    Ioctl(
        u8,
        Option<Box<DeviceInfo>>,
        Option<Box<DeviceInfo>>,
        Box<nix::Error>,
    ),

    /// An error returned when the response exceeds the maximum possible
    /// size of the ioctl buffer.
    IoctlResultTooLarge,

    /// An error returned on failure to get metadata for a device
    MetadataIo(PathBuf, String),

    /// An error returned on general IO failure
    GeneralIo(String),
}

impl fmt::Display for DmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContextInit(err) => {
                write!(f, "DM context not initialized due to IO error: {err}")
            }
            Self::InvalidArgument(err) => write!(f, "invalid argument: {err}"),
            Self::Ioctl(op, hdr_in, hdr_out, err) => write!(
                f,
                "low-level ioctl error due to nix error; ioctl number: {op}, input header: {hdr_in:?}, header result: {hdr_out:?}, error: {err}"
            ),
            Self::IoctlResultTooLarge => write!(
                f,
                "ioctl result too large for maximum buffer size: {} bytes",
                u32::MAX
            ),
            Self::MetadataIo(device_path, err) => write!(
                f,
                "failed to stat metadata for device at {} due to IO error: {}",
                device_path.display(),
                err
            ),
            Self::GeneralIo(err) => {
                write!(f, "failed to perform operation due to IO error: {err}")
            }
        }
    }
}

impl core::error::Error for DmError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Ioctl(_, _, _, err) => Some(err),
            _ => None,
        }
    }
}

/// Result specialization for DM functions.
pub type DmResult<S> = Result<S, DmError>;
