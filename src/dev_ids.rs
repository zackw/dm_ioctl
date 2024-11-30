// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Types for representing device IDs.
//!
//! A device-mapper device ID is either a "name" or a "uuid".
//! Both are required to be valid, non-empty C strings, consisting
//! entirely of ASCII characters, with a relatively short length limit
//! ([`DM_NAME_LEN`] and [`DM_UUID_LEN`], respectively; note that
//! these values _include_ the mandatory C-string terminator).
//!
//! Specific device-mapper targets may, or may not, apply further
//! restrictions to device IDs; note in particular that a "uuid" is
//! *not* necessarily required to be a well-formed Universally Unique
//! Identifier.

use core::{borrow::Borrow, fmt, ops::Deref};

use crate::bindings::{DM_NAME_LEN, DM_UUID_LEN};
use crate::errors::{DmError, DmResult};

#[cfg(test)]
#[path = "tests/dev_ids.rs"]
mod tests;

/// Returns an error if `value` does not meet the requirements for
/// a device ID whose length limit (including C-string terminator)
/// is `limit`.
fn check_id(value: &str, limit: usize) -> DmResult<()> {
    if value.is_empty() {
        return Err(DmError::DeviceIdEmpty);
    }
    if value.len() > limit - 1 {
        return Err(DmError::DeviceIdTooLong(limit - 1, value.len()));
    }
    if value.as_bytes().iter().any(|c| !(1u8..=127u8).contains(c)) {
        return Err(DmError::DeviceIdHasBadChars);
    }
    Ok(())
}

/// A borrowed string (analogous to [`str`]) that meets the
/// requirements for a device ID with length limit `LIMIT`.
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct DevIdStr<const LIMIT: usize> {
    inner: str,
}

/// An owned string (analogous to [`String`]) that meets the
/// requirements for a device ID with length limit `LIMIT`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct DevIdString<const LIMIT: usize> {
    inner: String,
}

/// A borrowed device name.
pub type DmName = DevIdStr<DM_NAME_LEN>;
/// An owned device name.
pub type DmNameBuf = DevIdString<DM_NAME_LEN>;

/// A borrowed device uuid.
pub type DmUuid = DevIdStr<DM_UUID_LEN>;
/// An owned device uuid.
pub type DmUuidBuf = DevIdString<DM_UUID_LEN>;

/// Used as a parameter for functions that take either a Device name
/// or a Device UUID.
#[derive(Debug, PartialEq, Eq)]
pub enum DevId<'a> {
    /// The parameter is the device's name
    Name(&'a DmName),
    /// The parameter is the device's devicemapper uuid
    Uuid(&'a DmUuid),
}

impl<'a> fmt::Display for DevId<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            DevId::Name(name) => write!(f, "{name}"),
            DevId::Uuid(uuid) => write!(f, "{uuid}"),
        }
    }
}

impl<const LIMIT: usize> DevIdStr<LIMIT> {
    /// Create a new borrowed `DevIdStr` from a `str` reference
    /// *without checking its validity*.
    /// SAFETY: Caller is responsible for doing the validity check first.
    unsafe fn new_unchecked(value: &str) -> &Self {
        // SAFETY: Converting &str to &DevIdStr<N> is safe because
        // DevIdStr is a repr(transparent) wrapper around str.
        // This "reborrow" construct is marginally more typesafe than
        // using mem::transmute.
        unsafe { &*(value as *const str as *const Self) }
    }

    /// Create a new borrowed `DevIdStr` from a `str` reference.
    pub fn new(value: &str) -> DmResult<&Self> {
        check_id(value, LIMIT)?;
        // SAFETY: We just did the validity check.
        Ok(unsafe { Self::new_unchecked(value) })
    }

    /// Get the inner value as bytes.
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }
}

impl<const LIMIT: usize> ToOwned for DevIdStr<LIMIT> {
    type Owned = DevIdString<LIMIT>;
    fn to_owned(&self) -> Self::Owned {
        DevIdString {
            inner: self.inner.to_owned(),
        }
    }
}

impl<const LIMIT: usize> fmt::Display for DevIdStr<LIMIT> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.inner)
    }
}

impl<const LIMIT: usize> DevIdString<LIMIT> {
    /// Construct a new owned identifier.
    pub fn new(value: String) -> DmResult<Self> {
        check_id(&value, LIMIT)?;
        Ok(DevIdString { inner: value })
    }
}

impl<const LIMIT: usize> AsRef<DevIdStr<LIMIT>> for DevIdString<LIMIT> {
    fn as_ref(&self) -> &DevIdStr<LIMIT> {
        self.deref()
    }
}

impl<const LIMIT: usize> Borrow<DevIdStr<LIMIT>> for DevIdString<LIMIT> {
    fn borrow(&self) -> &DevIdStr<LIMIT> {
        self.deref()
    }
}

impl<const LIMIT: usize> Deref for DevIdString<LIMIT> {
    type Target = DevIdStr<LIMIT>;
    fn deref(&self) -> &Self::Target {
        // SAFETY: The validity check was done when self was constructed.
        unsafe { DevIdStr::new_unchecked(&self.inner) }
    }
}
