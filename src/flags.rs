// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use bitflags::bitflags;

bitflags! {
    /// Flags used by devicemapper.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct DmFlags: u32 {
        /// In: If set, device should be made read-only.
        /// If cleared, device should be made read-write.
        ///
        /// Out: True if device is currently read-only.
        const DM_READONLY             = 1 << 0;

        /// In: If set, device should be suspended.
        /// If cleared, device should be resumed.
        ///
        /// Out: True if device is currently suspended.
        const DM_SUSPEND              = 1 << 1;

        // bit (1 << 2) is not used

        /// In: Use the passed-in minor number, don't allocate a new one.
        const DM_PERSISTENT_DEV       = 1 << 3;

        /// In: Retrieve table information rather than current status.
        /// (Only meaningful for `DM_DEV_STATUS`.)
        const DM_STATUS_TABLE         = 1 << 4;

        /// Out: True if an active table is present for this device.
        const DM_ACTIVE_PRESENT       = 1 << 5;

        /// Out: True if an inactive table is present for this device
        const DM_INACTIVE_PRESENT     = 1 << 6;

        /// Out: Indicates that the buffer passed in wasn't big enough
        /// for the results.
        const DM_BUFFER_FULL          = 1 << 8;

        /// In: Obsolete, ignored.
        const DM_SKIP_BDGET           = 1 << 9;

        /// In: When suspending a device, avoid attempting to freeze
        /// any filesystem backed by that device.
        const DM_SKIP_LOCKFS          = 1 << 10;

        /// In: When suspending a device, do not flush queued I/O first.
        ///
        /// May also avoid flushing queued I/O before waiting for
        /// "significant events" (`DM_DEV_WAIT`) or generating
        /// statistics (`DM_TABLE_STATUS`), depending on the target.
        const DM_NOFLUSH              = 1 << 11;

        /// In: Retrieve table information for the inactive table,
        /// rather than the active one.  Check the `DM_INACTIVE_PRESENT`
        /// bit before using the data returned; if it is cleared,
        /// there is no inactive table, and the information returned
        /// is garbage.
        const DM_QUERY_INACTIVE_TABLE = 1 << 12;

        /// Out: A uevent was generated, the caller may need to wait for it.
        const DM_UEVENT_GENERATED     = 1 << 13;

        /// In:
        ///
        /// For `DM_RENAME`: Change the UUID field, not the name field.
        /// Only permitted if no uuid was previously supplied.
        /// An existing uuid cannot be changed.
        ///
        /// For `DM_LIST_DEVICES`: Include UUIDs in the result.
        const DM_UUID                 = 1 << 14;

        /// In: Wipe all internal buffers before returning.
        /// Use when sending or requesting sensitive data, such as
        /// encryption keys.
        const DM_SECURE_DATA          = 1 << 15;

        /// Out: True if a message generated output data.
        const DM_DATA_OUT             = 1 << 16;

        /// In: Only meaningful with `DM_DEV_REMOVE` and `DM_REMOVE_ALL`.
        /// If set, devices that cannot be removed immediately because
        /// they are still in use should instead be scheduled for removal
        /// after all users are finished (e.g. filesystems unmounted).
        ///
        /// Out: Meaningful for all commands; true if the device has been
        /// scheduled to be removed after all users are finished.
        const DM_DEFERRED_REMOVE      = 1 << 17;

        /// Out: Device is suspended internally.
        const DM_INTERNAL_SUSPEND     = 1 << 18;

        /// In: Return the raw table information that would be measured
        /// by the IMA subsystem on device state change.
        const DM_IMA_MEASUREMENT      = 1 << 19;
    }

    /// Flags in `struct dm_name_list`'s extended portion.  We don't
    /// currently decode the extended portion but we may in the future.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct DmNameListFlags: u32 {
        /// This extended name record includes a UUID.
        const HAS_UUID           = 1;

        /// This extended name record does not include a UUID.
        ///
        /// (If UUIDs were requested, the kernel will set exactly
        /// one of HAS_UUID and DOESNT_HAVE_UUID in each record.
        /// If UUIDs were not requested, the kernel will set neither.
        /// This seems unnecessarily baroque to me, but to be frank
        /// if I had designed DM the entire API would be very different.)
        const DOESNT_HAVE_UUID   = 2;
    }
}
