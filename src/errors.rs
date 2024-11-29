// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The error types used by all public interfaces.

use core::fmt;
use std::io;

use crate::deviceinfo::DeviceInfo;

#[derive(Debug)]
/// Represents any kind of failure produced by this crate.
pub enum DmError {
    /// Unable to create a DM context due to a system-level error,
    /// e.g. not allowed to open `/dev/mapper/control`.
    ContextInit(io::Error),

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

    /// The kernel's response to a DM operation is impossibly large;
    /// so large that the `data_size` field of the `dm_ioctl` header
    /// cannot represent it.  This should never actually happen, as
    /// the kernel itself would not be able to generate a response
    /// that large.
    IoctlResultTooLarge,

    /// We were unable to construct a DM request packet due to a
    /// system-level error.
    RequestConstruction(io::Error),
}

impl fmt::Display for DmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContextInit(err) => {
                write!(f, "unable to initialize DM context: {err}")
            }
            Self::InvalidArgument(err) => write!(f, "invalid argument: {err}"),
            Self::Ioctl(op, hdr_in, hdr_out, err) => write!(
                f,
                "low-level ioctl error due to nix error; ioctl number: {op}, input header: {hdr_in:?}, header result: {hdr_out:?}, error: {err}"
            ),
            Self::IoctlResultTooLarge => write!(
                f,
                "ioctl result packet is impossibly large (probable bug)",
            ),
            Self::RequestConstruction(err) => {
                write!(f, "unable to construct ioctl request packet: {err}")
            }
        }
    }
}

impl core::error::Error for DmError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::ContextInit(err) => Some(err),
            Self::Ioctl(_, _, _, err) => Some(err),
            Self::RequestConstruction(err) => Some(err),
            _ => None,
        }
    }
}

/// Result specialization for DM functions.
pub type DmResult<S> = Result<S, DmError>;
