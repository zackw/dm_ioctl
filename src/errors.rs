// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The error types used by all public interfaces.

use core::fmt;
use std::io;

use crate::deviceinfo::DeviceInfo;
use crate::ioctl_cmds::DmIoctlCmd;

#[derive(Debug)]
#[non_exhaustive]
/// Represents any kind of failure produced by this crate.
pub enum DmError {
    /// Unable to create a DM context due to a system-level error,
    /// e.g. not allowed to open `/dev/mapper/control`.
    ContextInit(io::Error),

    /// The empty string was provided as a device ID argument.
    DeviceIdEmpty,

    /// A device ID argument was too long.  The fields are the
    /// length limit and the length of the argument, in that order.
    DeviceIdTooLong(usize, usize),

    /// A device ID argument contains characters that cannot be used
    /// in device IDs.
    DeviceIdHasBadChars,

    /// A DM ioctl operation returned a system-level error.  Records
    /// the opcode, the system error code, and, if possible, decoded
    /// versions of the request and response packets, to facilitate
    /// debugging.
    Ioctl(
        DmIoctlCmd,
        Option<Box<DeviceInfo>>,
        Option<Box<DeviceInfo>>,
        nix::Error,
    ),

    /// The kernel's response to a DM operation was malformed in
    /// some way; the string provides details.
    IoctlResultMalformed(&'static str),

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
            Self::DeviceIdEmpty => {
                write!(f, "device ID cannot be the empty string")
            },
            Self::DeviceIdTooLong(limit, actual) => {
                write!(f, "device ID is too long ({actual} > {limit} bytes)")
            },
            Self::DeviceIdHasBadChars => {
                write!(f, "device ID contains NULs or non-ASCII chars")
            }
            Self::Ioctl(op, hdr_in, hdr_out, err) => write!(
                f,
                "DM operation {op:?} failed: input header: {hdr_in:?}, header result: {hdr_out:?}, error: {err}"
            ),
            Self::IoctlResultMalformed(detail) => write!(
                f,
                "ioctl result packet is malformed (kernel bug?): {detail}"
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
