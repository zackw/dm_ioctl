// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// `_IOC` group code for device mapper ioctls.
pub const DM_IOCTL_GROUP: u32 = 0xFD;

/// `_IOC` operation codes for device mapper ioctls.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(non_camel_case_types)]
#[non_exhaustive]
pub enum DmIoctlCmd {
    /// Get the version information for the ioctl interface.
    DM_VERSION = 0,

    ///  Remove all dm devices, destroy all tables.  Only really used for debug.
    DM_REMOVE_ALL = 1,

    /// Get a list of all the dm device names.
    DM_LIST_DEVICES = 2,

    /// Create a new device, neither the 'active' or 'inactive' table
    /// slots will be filled.  The device will be in suspended state
    /// after creation, however any io to the device will get errored
    /// since it will be out-of-bounds.
    DM_DEV_CREATE = 3,

    /// Remove a device, destroy any tables.
    DM_DEV_REMOVE = 4,

    /// Rename a device or set its uuid if none was previously supplied.
    DM_DEV_RENAME = 5,

    /// This performs both suspend and resume, depending which flag is
    /// passed in.
    ///
    /// Suspend: This command will not return until all pending io to
    /// the device has completed.  Further io will be deferred until
    /// the device is resumed.
    ///
    /// Resume: It is no longer an error to issue this command on an
    /// unsuspended device.  If a table is present in the 'inactive'
    /// slot, it will be moved to the active slot, then the old table
    /// from the active slot will be _destroyed_.  Finally the device
    /// is resumed.
    DM_DEV_SUSPEND = 6,

    /// Retrieves the status for the table in the 'active' slot.
    DM_DEV_STATUS = 7,

    /// Wait for a significant event to occur to the device.  This
    /// could either be caused by an event triggered by one of the
    /// targets of the table in the 'active' slot, or a table change.
    DM_DEV_WAIT = 8,

    /// Load a table into the 'inactive' slot for the device.  The
    /// device does _not_ need to be suspended prior to this command.
    DM_TABLE_LOAD = 9,

    /// Destroy any table in the 'inactive' slot (ie. abort).
    DM_TABLE_CLEAR = 10,

    /// Return a set of device dependencies for the 'active' table.
    DM_TABLE_DEPS = 11,

    /// Return the targets status for the 'active' table.
    DM_TABLE_STATUS = 12,

    /// ???
    DM_LIST_VERSIONS = 13,

    /// Pass a message string to the target at a specific offset of a device.
    DM_TARGET_MSG = 14,

    /// Set the geometry of a device by passing in a string in this format:
    ///
    /// "cylinders heads sectors_per_track start_sector"
    ///
    /// Beware that CHS geometry is nearly obsolete and only provided
    /// for compatibility with dm devices that can be booted by a PC
    /// BIOS.  See struct hd_geometry for range limits.  Also note that
    /// the geometry is erased if the device size changes.
    DM_DEV_SET_GEOMETRY = 15,

    /// ???
    DM_DEV_ARM_POLL = 16,

    /// ???
    DM_GET_TARGET_VERSION = 17,
}

// Map device-mapper ioctl commands to (major, minor, patchlevel)
// tuple specifying the required kernel ioctl interface version.
pub(crate) fn ioctl_to_version(ioctl: DmIoctlCmd) -> (u32, u32, u32) {
    use DmIoctlCmd::*;
    match ioctl {
        DM_VERSION => (4, 0, 0),
        DM_REMOVE_ALL => (4, 0, 0),
        DM_LIST_DEVICES => (4, 0, 0),
        DM_DEV_CREATE => (4, 0, 0),
        DM_DEV_REMOVE => (4, 0, 0),
        DM_DEV_RENAME => (4, 0, 0),
        DM_DEV_SUSPEND => (4, 0, 0),
        DM_DEV_STATUS => (4, 0, 0),
        DM_DEV_WAIT => (4, 0, 0),
        DM_TABLE_LOAD => (4, 0, 0),
        DM_TABLE_CLEAR => (4, 0, 0),
        DM_TABLE_DEPS => (4, 0, 0),
        DM_TABLE_STATUS => (4, 0, 0),
        DM_LIST_VERSIONS => (4, 1, 0),
        DM_TARGET_MSG => (4, 2, 0),
        DM_DEV_SET_GEOMETRY => (4, 6, 0),
        // libdevmapper sets DM_DEV_ARM_POLL to (4, 36, 0) however the
        // command was added after 4.36.0: depend on 4.37 to reliably
        // access ARM_POLL.
        DM_DEV_ARM_POLL => (4, 37, 0),
        DM_GET_TARGET_VERSION => (4, 41, 0),
    }
}
