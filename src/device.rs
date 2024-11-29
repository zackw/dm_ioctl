// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{fmt, path::Path, str::FromStr};

use nix::libc::{dev_t, major, makedev, minor};
use nix::sys::stat::{self, SFlag};

use crate::errors::{DmError, DmResult};

#[cfg(test)]
#[path = "tests/device.rs"]
mod test;

/// A struct containing the device's major and minor numbers
///
/// Also allows conversion to/from a single 64bit dev_t value.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Device {
    /// Device major number
    pub major: u32,
    /// Device minor number
    pub minor: u32,
}

/// Display format is the device number in `<major>:<minor>` format
impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.major, self.minor)
    }
}

impl FromStr for Device {
    type Err = DmError;

    fn from_str(s: &str) -> Result<Device, DmError> {
        let vals = s.split(':').collect::<Vec<_>>();
        if vals.len() != 2 {
            let err_msg = format!("value \"{s}\" split into wrong number of fields");
            return Err(DmError::InvalidArgument(err_msg));
        }
        let major = vals[0].parse::<u32>().map_err(|_| {
            DmError::InvalidArgument(format!(
                "could not parse \"{}\" to obtain major number",
                vals[0]
            ))
        })?;
        let minor = vals[1].parse::<u32>().map_err(|_| {
            DmError::InvalidArgument(format!(
                "could not parse \"{}\" to obtain minor number",
                vals[1]
            ))
        })?;
        Ok(Device { major, minor })
    }
}

impl From<dev_t> for Device {
    fn from(val: dev_t) -> Device {
        let major = unsafe { major(val) };

        let minor = unsafe { minor(val) };

        Device { major, minor }
    }
}

impl From<Device> for dev_t {
    fn from(dev: Device) -> dev_t {
        makedev(dev.major, dev.minor)
    }
}

/// The Linux kernel's kdev_t encodes major/minor values as mmmM MMmm.
impl Device {
    /// Make a Device from a kdev_t.
    pub fn from_kdev_t(val: u32) -> Device {
        Device {
            major: (val & 0xf_ff00) >> 8,
            minor: (val & 0xff) | ((val >> 12) & 0xf_ff00),
        }
    }

    /// Convert to a kdev_t. Return None if values are not expressible as a
    /// kdev_t.
    pub fn to_kdev_t(self) -> Option<u32> {
        if self.major > 0xfff || self.minor > 0xf_ffff {
            return None;
        }

        Some((self.minor & 0xff) | (self.major << 8) | ((self.minor & !0xff) << 12))
    }
}

/// Get a device number from a device node.
/// Return None if the device is not a block device; devicemapper is not
/// interested in other sorts of devices. Return None if the device appears
/// not to exist.
pub fn devnode_to_devno(path: &Path) -> DmResult<Option<u64>> {
    match stat::stat(path) {
        Ok(metadata) => Ok(
            if metadata.st_mode & SFlag::S_IFMT.bits() == SFlag::S_IFBLK.bits() {
                Some(metadata.st_rdev)
            } else {
                None
            },
        ),
        Err(nix::Error::ENOENT) => Ok(None),
        Err(err) => Err(DmError::MetadataIo(path.to_owned(), err.to_string())),
    }
}
