// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Types for working with device numbers in the format expected by
//! the Linux kernel.
//!
//! The kernel works with `kdev_t`, which is a `u32` quantity divided
//! into a 12-bit "major number" and a 20-bit "minor number".  In some
//! cases—notably, for our purposes, the `dev` fields of
//! [`dm_ioctl`][crate::bindings::dm_ioctl],
//! [`dm_target_deps`][crate::bindings::dm_target_deps], and
//! [`dm_name_list`][crate::bindings::dm_name_list]—a `kdev_t`
//! quantity is passed to user space in a 64-bit field, but its
//! high 32 bits are reserved, should-be-zero.
//!
//! For backward compatibility with very old kernels where `kdev_t`
//! was only 16 bits, the major and minor numbers are not contiguous,
//! but rather are packed into the 32-bit field as `mnoM NOpq` where
//! `mnopq` are hex digits of the minor number and `MNO` are hex
//! digits of the major number.
//!
//! GNU libc extended this format to 64 bits, with 32 bits each for
//! major and minor numbers, using the pattern `MNOP Qmno pqrR STst`.
//! musl and bionic libc adopted the same 64-bit format.  If the
//! kernel ever starts using wider device numbers, one would hope
//! it would also follow suit.  Therefore, when decoding the above
//! 64-bit fields from the kernel, we use the C library's extended
//! format, but when encoding a kdev_t from a Device object, we
//! produce a 32-bit quantity or fail.

use core::fmt;

#[cfg(test)]
#[path = "tests/device.rs"]
mod test;

/// A struct representing a block device, identified by major and
/// minor numbers.
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

impl Device {
    /// Make a `Device` from a 64-bit extended `kdev_t`.
    /// See module-level documentation for discussion of the format.
    #[rustfmt::skip]
    #[allow(clippy::identity_op)]
    pub fn from_kdev_t(val: u64) -> Device {
        let major: u32 =
            (((val & 0x0000_0000_000f_ff00_u64) >>  8) as u32)
          | (((val & 0xffff_f000_0000_0000_u64) >> 32) as u32);

        let minor: u32 =
            (((val & 0x0000_0000_0000_00ff_u64) >>  0) as u32)
          | (((val & 0x0000_0fff_fff0_0000_u64) >> 12) as u32);

        Device { major, minor }
    }

    /// Convert self to a `kdev_t` value.  Returns `None` if self
    /// is not representable as a *32-bit* kdev_t.
    pub fn to_kdev_t(self) -> Option<u32> {
        if self.major > 0x0fff || self.minor > 0xf_ffff {
            return None;
        }

        let major = self.major << 8;
        let minor = (self.minor & 0xff) | ((self.minor & 0xf_ff00) << 12);
        Some(major | minor)
    }
}
