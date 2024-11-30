// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// `_IOC` type code for device mapper ioctls.
pub const DM_IOCTL: u32 = 0xFD;

/// `_IOC` operation codes for device mapper ioctls.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum DmIoctlCmd {
    /// Get the version information for the ioctl interface.
    DM_VERSION_CMD = 0,

    ///  Remove all dm devices, destroy all tables.  Only really used for debug.
    DM_REMOVE_ALL_CMD = 1,

    /// Get a list of all the dm device names.
    DM_LIST_DEVICES_CMD = 2,

    /// Create a new device, neither the 'active' or 'inactive' table
    /// slots will be filled.  The device will be in suspended state
    /// after creation, however any io to the device will get errored
    /// since it will be out-of-bounds.
    DM_DEV_CREATE_CMD = 3,

    /// Remove a device, destroy any tables.
    DM_DEV_REMOVE_CMD = 4,

    /// Rename a device or set its uuid if none was previously supplied.
    DM_DEV_RENAME_CMD = 5,

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
    DM_DEV_SUSPEND_CMD = 6,

    /// Retrieves the status for the table in the 'active' slot.
    DM_DEV_STATUS_CMD = 7,

    /// Wait for a significant event to occur to the device.  This
    /// could either be caused by an event triggered by one of the
    /// targets of the table in the 'active' slot, or a table change.
    DM_DEV_WAIT_CMD = 8,

    /// Load a table into the 'inactive' slot for the device.  The
    /// device does _not_ need to be suspended prior to this command.
    DM_TABLE_LOAD_CMD = 9,

    /// Destroy any table in the 'inactive' slot (ie. abort).
    DM_TABLE_CLEAR_CMD = 10,

    /// Return a set of device dependencies for the 'active' table.
    DM_TABLE_DEPS_CMD = 11,

    /// Return the targets status for the 'active' table.
    DM_TABLE_STATUS_CMD = 12,

    /// ???
    DM_LIST_VERSIONS_CMD = 13,

    /// Pass a message string to the target at a specific offset of a device.
    DM_TARGET_MSG_CMD = 14,

    /// Set the geometry of a device by passing in a string in this format:
    ///
    /// "cylinders heads sectors_per_track start_sector"
    ///
    /// Beware that CHS geometry is nearly obsolete and only provided
    /// for compatibility with dm devices that can be booted by a PC
    /// BIOS.  See struct hd_geometry for range limits.  Also note that
    /// the geometry is erased if the device size changes.
    DM_DEV_SET_GEOMETRY_CMD = 15,

    /// ???
    DM_DEV_ARM_POLL_CMD = 16,

    /// ???
    DM_GET_TARGET_VERSION_CMD = 17,
}

pub use DmIoctlCmd::*;

// Map device-mapper ioctl commands to (major, minor, patchlevel)
// tuple specifying the required kernel ioctl interface version.
pub(crate) fn ioctl_to_version(ioctl: DmIoctlCmd) -> (u32, u32, u32) {
    match ioctl {
        DM_VERSION_CMD => (4, 0, 0),
        DM_REMOVE_ALL_CMD => (4, 0, 0),
        DM_LIST_DEVICES_CMD => (4, 0, 0),
        DM_DEV_CREATE_CMD => (4, 0, 0),
        DM_DEV_REMOVE_CMD => (4, 0, 0),
        DM_DEV_RENAME_CMD => (4, 0, 0),
        DM_DEV_SUSPEND_CMD => (4, 0, 0),
        DM_DEV_STATUS_CMD => (4, 0, 0),
        DM_DEV_WAIT_CMD => (4, 0, 0),
        DM_TABLE_LOAD_CMD => (4, 0, 0),
        DM_TABLE_CLEAR_CMD => (4, 0, 0),
        DM_TABLE_DEPS_CMD => (4, 0, 0),
        DM_TABLE_STATUS_CMD => (4, 0, 0),
        DM_LIST_VERSIONS_CMD => (4, 1, 0),
        DM_TARGET_MSG_CMD => (4, 2, 0),
        DM_DEV_SET_GEOMETRY_CMD => (4, 6, 0),
        // libdevmapper sets DM_DEV_ARM_POLL to (4, 36, 0) however the
        // command was added after 4.36.0: depend on 4.37 to reliably
        // access ARM_POLL.
        DM_DEV_ARM_POLL_CMD => (4, 37, 0),
        DM_GET_TARGET_VERSION_CMD => (4, 41, 0),
    }
}
