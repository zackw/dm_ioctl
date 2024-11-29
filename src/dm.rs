// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{
    cmp,
    fs::File,
    io::{Cursor, Read, Write},
    mem::size_of,
    os::unix::io::{AsRawFd, RawFd},
    slice, str,
};

use nix::libc::ioctl as nix_ioctl;
use semver::Version;

use crate::{
    device::Device,
    deviceinfo::DeviceInfo,
    dm_flags::DmFlags,
    dm_ioctl as dmi,
    errors::{DmError, DmResult},
    types::{DevId, DmName, DmNameBuf, DmUuid},
    util::{
        align_to, c_struct_from_slice, mut_slice_from_c_str, slice_from_c_struct,
        str_from_byte_slice, str_from_c_str,
    },
};

#[cfg(test)]
#[path = "tests/dm.rs"]
mod tests;

/// Control path for user space to pass IOCTL to kernel DM
const DM_CTL_PATH: &str = "/dev/mapper/control";

/// Start with a large buffer to make BUFFER_FULL rare. Libdm does this too.
const MIN_BUF_SIZE: usize = 16 * 1024;

/// Context needed for communicating with devicemapper.
pub struct DM {
    file: File,
}

impl DmFlags {
    /// Generate a header to be used for IOCTL.
    fn to_ioctl_hdr(
        self,
        id: Option<&DevId<'_>>,
        allowable_flags: DmFlags,
    ) -> DmResult<dmi::Struct_dm_ioctl> {
        let clean_flags = allowable_flags & self;
        let mut hdr: dmi::Struct_dm_ioctl = crate::bindings::dm_ioctl {
            flags: clean_flags.bits(),
            event_nr: 0,
            data_start: size_of::<dmi::Struct_dm_ioctl>() as u32,
            ..Default::default()
        };

        if let Some(id) = id {
            match id {
                DevId::Name(name) => DM::hdr_set_name(&mut hdr, name)?,
                DevId::Uuid(uuid) => DM::hdr_set_uuid(&mut hdr, uuid)?,
            };
        };

        Ok(hdr)
    }
}

impl DM {
    /// Create a new context for communicating with DM.
    pub fn new() -> DmResult<DM> {
        Ok(DM {
            file: File::open(DM_CTL_PATH).map_err(|err| DmError::ContextInit(err.to_string()))?,
        })
    }

    fn hdr_set_name(hdr: &mut dmi::Struct_dm_ioctl, name: &DmName) -> DmResult<()> {
        let _ = name
            .as_bytes()
            .read(mut_slice_from_c_str(&mut hdr.name))
            .map_err(|err| DmError::GeneralIo(err.to_string()))?;
        Ok(())
    }

    fn hdr_set_uuid(hdr: &mut dmi::Struct_dm_ioctl, uuid: &DmUuid) -> DmResult<()> {
        let _ = uuid
            .as_bytes()
            .read(mut_slice_from_c_str(&mut hdr.uuid))
            .map_err(|err| DmError::GeneralIo(err.to_string()))?;
        Ok(())
    }

    /// Get the file within the DM context, likely for polling purposes.
    pub fn file(&self) -> &File {
        &self.file
    }

    // Make the ioctl call specified by the given ioctl number.
    // Set the required DM version to the lowest that supports the given ioctl.
    fn do_ioctl(
        &self,
        ioctl: dmi::DmIoctlCmd,
        hdr: &mut dmi::Struct_dm_ioctl,
        in_data: Option<&[u8]>,
    ) -> DmResult<(DeviceInfo, Vec<u8>)> {
        let op = request_code_readwrite!(dmi::DM_IOCTL, ioctl, size_of::<dmi::Struct_dm_ioctl>());

        let ioctl_version = dmi::ioctl_to_version(ioctl);
        hdr.version[0] = ioctl_version.0;
        hdr.version[1] = ioctl_version.1;
        hdr.version[2] = ioctl_version.2;

        let data_size = cmp::max(
            MIN_BUF_SIZE,
            size_of::<dmi::Struct_dm_ioctl>() + in_data.map_or(0, |x| x.len()),
        );

        let mut buffer: Vec<u8> = Vec::with_capacity(data_size);
        let mut buffer_hdr;
        loop {
            hdr.data_size = buffer.capacity() as u32;

            let hdr_slc = unsafe {
                let len = hdr.data_start as usize;
                let ptr = hdr as *mut dmi::Struct_dm_ioctl as *mut u8;
                slice::from_raw_parts_mut(ptr, len)
            };

            buffer.clear();
            buffer.extend_from_slice(hdr_slc);
            if let Some(in_data) = in_data {
                buffer.extend(in_data.iter().cloned());
            }
            buffer.resize(buffer.capacity(), 0);

            buffer_hdr = unsafe { &mut *(buffer.as_mut_ptr() as *mut dmi::Struct_dm_ioctl) };

            if let Err(err) = unsafe {
                convert_ioctl_res!(nix_ioctl(self.file.as_raw_fd(), op, buffer.as_mut_ptr()))
            } {
                return Err(DmError::Ioctl(
                    op as u8,
                    DeviceInfo::new(*hdr).ok().map(Box::new),
                    DeviceInfo::new(*buffer_hdr).ok().map(Box::new),
                    Box::new(err),
                ));
            }

            if (buffer_hdr.flags & DmFlags::DM_BUFFER_FULL.bits()) == 0 {
                break;
            }

            // If DM_BUFFER_FULL is set, DM requires more space for the
            // response.  Double the capacity of the buffer and re-try the
            // ioctl. If the size of the buffer is already as large as can be
            // possibly expressed in data_size field, return an error.
            // Never allow the size to exceed u32::MAX.
            let len = buffer.capacity();
            if len == u32::MAX as usize {
                return Err(DmError::IoctlResultTooLarge);
            }
            buffer.resize((len as u32).saturating_mul(2) as usize, 0);
        }

        let data_end = cmp::max(buffer_hdr.data_size, buffer_hdr.data_start);

        Ok((
            DeviceInfo::try_from(*buffer_hdr)?,
            buffer[buffer_hdr.data_start as usize..data_end as usize].to_vec(),
        ))
    }

    /// Devicemapper version information: Major, Minor, and patchlevel versions.
    pub fn version(&self) -> DmResult<(u32, u32, u32)> {
        let mut hdr = DmFlags::default().to_ioctl_hdr(None, DmFlags::empty())?;

        let (hdr_out, _) = self.do_ioctl(dmi::DM_VERSION_CMD, &mut hdr, None)?;

        Ok((
            hdr_out
                .version()
                .major
                .try_into()
                .expect("dm_ioctl struct field is u32"),
            hdr_out
                .version()
                .minor
                .try_into()
                .expect("dm_ioctl struct field is u32"),
            hdr_out
                .version()
                .patch
                .try_into()
                .expect("dm_ioctl struct field is u32"),
        ))
    }

    /// Remove all DM devices and tables. Use discouraged other than
    /// for debugging.
    ///
    /// If `DM_DEFERRED_REMOVE` is set, the request will succeed for
    /// in-use devices, and they will be removed when released.
    ///
    /// Valid flags: `DM_DEFERRED_REMOVE`
    pub fn remove_all(&self, flags: DmFlags) -> DmResult<()> {
        let mut hdr = flags.to_ioctl_hdr(None, DmFlags::DM_DEFERRED_REMOVE)?;

        self.do_ioctl(dmi::DM_REMOVE_ALL_CMD, &mut hdr, None)?;

        Ok(())
    }

    /// Returns a list of tuples containing DM device names, a Device, which
    /// holds their major and minor device numbers, and on kernels that
    /// support it, each device's last event_nr.
    pub fn list_devices(&self) -> DmResult<Vec<(DmNameBuf, Device, Option<u32>)>> {
        let mut hdr = DmFlags::default().to_ioctl_hdr(None, DmFlags::empty())?;
        let (hdr_out, data_out) = self.do_ioctl(dmi::DM_LIST_DEVICES_CMD, &mut hdr, None)?;

        let event_nr_set = hdr_out.version() >= &Version::new(4, 37, 0);

        let mut devs = Vec::new();
        if !data_out.is_empty() {
            let mut result = &data_out[..];

            loop {
                let device =
                    c_struct_from_slice::<dmi::Struct_dm_name_list>(result).ok_or_else(|| {
                        DmError::InvalidArgument("Received null pointer from kernel".to_string())
                    })?;
                let name_offset = unsafe {
                    (device.name.as_ptr() as *const u8).offset_from(device as *const _ as *const u8)
                } as usize;

                let dm_name = str_from_byte_slice(&result[name_offset..])
                    .map(|s| s.to_owned())
                    .ok_or_else(|| {
                        DmError::InvalidArgument("Devicemapper name is not valid UTF8".to_string())
                    })?;

                // Get each device's event number after its name, if the kernel
                // DM version supports it.
                // Should match offset calc in kernel's
                // drivers/md/dm-ioctl.c:list_devices
                let event_nr = if event_nr_set {
                    // offsetof "name" in Struct_dm_name_list.
                    let offset = align_to(name_offset + dm_name.len() + 1, size_of::<u64>());
                    let nr = u32::from_ne_bytes(
                        result[offset..offset + size_of::<u32>()]
                            .try_into()
                            .map_err(|_| {
                                DmError::InvalidArgument(
                                    "Incorrectly sized slice for u32".to_string(),
                                )
                            })?,
                    );

                    Some(nr)
                } else {
                    None
                };

                devs.push((DmNameBuf::new(dm_name)?, device.dev.into(), event_nr));

                if device.next == 0 {
                    break;
                }

                result = &result[device.next as usize..];
            }
        }

        Ok(devs)
    }

    /// Create a DM device. It starts out in a "suspended" state.
    ///
    /// Valid flags: `DM_READONLY`, `DM_PERSISTENT_DEV`
    ///
    /// # Example
    ///
    /// ```no_run
    /// use devicemapper::{DM, DmFlags, DmName};
    ///
    /// let dm = DM::new().unwrap();
    ///
    /// // Setting a uuid is optional
    /// let name = DmName::new("example-dev").expect("is valid DM name");
    /// let dev = dm.device_create(name, None, DmFlags::default()).unwrap();
    /// ```
    pub fn device_create(
        &self,
        name: &DmName,
        uuid: Option<&DmUuid>,
        flags: DmFlags,
    ) -> DmResult<DeviceInfo> {
        let mut hdr =
            flags.to_ioctl_hdr(None, DmFlags::DM_READONLY | DmFlags::DM_PERSISTENT_DEV)?;

        Self::hdr_set_name(&mut hdr, name)?;
        if let Some(uuid) = uuid {
            Self::hdr_set_uuid(&mut hdr, uuid)?;
        }

        self.do_ioctl(dmi::DM_DEV_CREATE_CMD, &mut hdr, None)
            .map(|(hdr, _)| hdr)
    }

    /// Remove a DM device and its mapping tables.
    ///
    /// If `DM_DEFERRED_REMOVE` is set, the request for an in-use
    /// devices will succeed, and it will be removed when no longer
    /// used.
    ///
    /// Valid flags: `DM_DEFERRED_REMOVE`
    pub fn device_remove(&self, id: &DevId<'_>, flags: DmFlags) -> DmResult<DeviceInfo> {
        let mut hdr = flags.to_ioctl_hdr(Some(id), DmFlags::DM_DEFERRED_REMOVE)?;
        self.do_ioctl(dmi::DM_DEV_REMOVE_CMD, &mut hdr, None)
            .map(|(hdr, _)| hdr)
    }

    /// Change a DM device's name OR set the device's uuid for the first time.
    ///
    /// Prerequisite: if `new == DevId::Name(new_name)`, `old_name != new_name`
    /// Prerequisite: if `new == DevId::Uuid(uuid)`, device's current uuid
    /// must be `""`.
    /// Note: Possibly surprisingly, returned `DeviceInfo`'s uuid or name field
    /// contains the previous value, not the newly set value.
    pub fn device_rename(&self, old_name: &DmName, new: &DevId<'_>) -> DmResult<DeviceInfo> {
        let (flags, id_in) = match *new {
            DevId::Name(name) => (DmFlags::default(), name.as_bytes()),
            DevId::Uuid(uuid) => (DmFlags::DM_UUID, uuid.as_bytes()),
        };

        let data_in = [id_in, b"\0"].concat();

        let mut hdr = flags.to_ioctl_hdr(None, DmFlags::DM_UUID)?;
        Self::hdr_set_name(&mut hdr, old_name)?;

        self.do_ioctl(dmi::DM_DEV_RENAME_CMD, &mut hdr, Some(&data_in))
            .map(|(hdr, _)| hdr)
    }

    /// Suspend or resume a DM device, depending on if `DM_SUSPEND` flag
    /// is set or not.
    ///
    /// Resuming a DM device moves a table loaded into the "inactive"
    /// slot by [`Self::table_load`] into the "active" slot.
    ///
    /// Will block until pending I/O is completed unless DM_NOFLUSH
    /// flag is given. Will freeze filesystem unless DM_SKIP_LOCKFS
    /// flags is given. Additional I/O to a suspended device will be
    /// held until it is resumed.
    ///
    /// Valid flags: `DM_SUSPEND`, `DM_NOFLUSH`, `DM_SKIP_LOCKFS`
    ///
    /// # Example
    ///
    /// ```no_run
    /// use devicemapper::{DM, DevId, DmFlags, DmName};
    /// let dm = DM::new().unwrap();
    ///
    /// let name = DmName::new("example-dev").expect("is valid DM name");
    /// let id = DevId::Name(name);
    /// dm.device_suspend(&id, DmFlags::DM_SUSPEND).unwrap();
    /// ```
    pub fn device_suspend(&self, id: &DevId<'_>, flags: DmFlags) -> DmResult<DeviceInfo> {
        let mut hdr = flags.to_ioctl_hdr(
            Some(id),
            DmFlags::DM_SUSPEND | DmFlags::DM_NOFLUSH | DmFlags::DM_SKIP_LOCKFS,
        )?;

        self.do_ioctl(dmi::DM_DEV_SUSPEND_CMD, &mut hdr, None)
            .map(|(hdr, _)| hdr)
    }

    /// Get DeviceInfo for a device. This is also returned by other
    /// methods, but if just the DeviceInfo is desired then this just
    /// gets it.
    pub fn device_info(&self, id: &DevId<'_>) -> DmResult<DeviceInfo> {
        let mut hdr = DmFlags::default().to_ioctl_hdr(Some(id), DmFlags::empty())?;

        self.do_ioctl(dmi::DM_DEV_STATUS_CMD, &mut hdr, None)
            .map(|(hdr, _)| hdr)
    }

    /// Wait for a device to report an event.
    ///
    /// Once an event occurs, this function behaves just like
    /// [`Self::table_status`], see that function for more details.
    ///
    /// This interface is not very friendly to monitoring multiple devices.
    /// Events are also exported via uevents, that method may be preferable.
    #[allow(clippy::type_complexity)]
    pub fn device_wait(
        &self,
        id: &DevId<'_>,
        flags: DmFlags,
    ) -> DmResult<(DeviceInfo, Vec<(u64, u64, String, String)>)> {
        let mut hdr = flags.to_ioctl_hdr(Some(id), DmFlags::DM_QUERY_INACTIVE_TABLE)?;

        let (hdr_out, data_out) = self.do_ioctl(dmi::DM_DEV_WAIT_CMD, &mut hdr, None)?;

        let status = DM::parse_table_status(hdr.target_count, &data_out)?;

        Ok((hdr_out, status))
    }

    /// Load targets for a device into its inactive table slot.
    ///
    /// `targets` is an array of `(sector_start, sector_length, type, params)`.
    ///
    /// `flags` Valid flags: `DM_READ_ONLY`, `DM_SECURE_DATA`
    ///
    /// # Example
    ///
    /// ```no_run
    /// use devicemapper::{DM, DevId, DmName, DmFlags};
    /// let dm = DM::new().unwrap();
    ///
    /// // Create a 16MiB device (32768 512-byte sectors) that maps to /dev/sdb1
    /// // starting 1MiB into sdb1
    /// let table = vec![(
    ///     0,
    ///     32768,
    ///     "linear".into(),
    ///     "/dev/sdb1 2048".into()
    /// )];
    ///
    /// let name = DmName::new("example-dev").expect("is valid DM name");
    /// let id = DevId::Name(name);
    /// dm.table_load(&id, &table, DmFlags::default()).unwrap();
    /// ```
    pub fn table_load(
        &self,
        id: &DevId<'_>,
        targets: &[(u64, u64, String, String)],
        flags: DmFlags,
    ) -> DmResult<DeviceInfo> {
        let mut cursor = Cursor::new(Vec::new());

        // Construct targets first, since we need to know how many & size
        // before initializing the header.
        for (sector_start, length, target_type, params) in targets {
            let mut targ = dmi::Struct_dm_target_spec {
                sector_start: *sector_start,
                length: *length,
                status: 0,
                ..Default::default()
            };

            let dst = mut_slice_from_c_str(&mut targ.target_type);
            assert!(
                target_type.len() <= dst.len(),
                "TargetType max length = targ.target_type.len()"
            );
            let _ = target_type
                .as_bytes()
                .read(dst)
                .map_err(|err| DmError::GeneralIo(err.to_string()))?;

            // Size of the largest single member of dm_target_spec
            let align_to_size = size_of::<u64>();
            let aligned_len = align_to(params.len() + 1usize, align_to_size);
            targ.next = (size_of::<dmi::Struct_dm_target_spec>() + aligned_len) as u32;

            cursor
                .write_all(slice_from_c_struct(&targ))
                .map_err(|err| DmError::GeneralIo(err.to_string()))?;
            cursor
                .write_all(params.as_bytes())
                .map_err(|err| DmError::GeneralIo(err.to_string()))?;

            let padding = aligned_len - params.len();
            cursor
                .write_all(vec![0; padding].as_slice())
                .map_err(|err| DmError::GeneralIo(err.to_string()))?;
        }

        let mut hdr =
            flags.to_ioctl_hdr(Some(id), DmFlags::DM_READONLY | DmFlags::DM_SECURE_DATA)?;

        // io_ioctl() will set hdr.data_size but we must set target_count
        hdr.target_count = targets.len() as u32;

        // Flatten targets into a buf
        let data_in = cursor.into_inner();

        self.do_ioctl(dmi::DM_TABLE_LOAD_CMD, &mut hdr, Some(&data_in))
            .map(|(hdr, _)| hdr)
    }

    /// Clear the "inactive" table for a device.
    pub fn table_clear(&self, id: &DevId<'_>) -> DmResult<DeviceInfo> {
        let mut hdr = DmFlags::default().to_ioctl_hdr(Some(id), DmFlags::empty())?;

        self.do_ioctl(dmi::DM_TABLE_CLEAR_CMD, &mut hdr, None)
            .map(|(hdr, _)| hdr)
    }

    /// Query DM for which devices are referenced by the "active"
    /// table for this device.
    ///
    /// If DM_QUERY_INACTIVE_TABLE is set, instead return for the
    /// inactive table.
    ///
    /// Valid flags: DM_QUERY_INACTIVE_TABLE
    pub fn table_deps(&self, id: &DevId<'_>, flags: DmFlags) -> DmResult<Vec<Device>> {
        let mut hdr = flags.to_ioctl_hdr(Some(id), DmFlags::DM_QUERY_INACTIVE_TABLE)?;

        let (_, data_out) = self.do_ioctl(dmi::DM_TABLE_DEPS_CMD, &mut hdr, None)?;

        if data_out.is_empty() {
            Ok(vec![])
        } else {
            let result = &data_out[..];
            let target_deps = unsafe { &*(result.as_ptr() as *const dmi::Struct_dm_target_deps) };

            let dev_slc = unsafe {
                slice::from_raw_parts(
                    result[size_of::<dmi::Struct_dm_target_deps>()..].as_ptr() as *const u64,
                    target_deps.count as usize,
                )
            };

            // Note: The DM target_deps struct reserves 64 bits for each entry
            // but only 32 bits is used by kernel "huge" dev_t encoding.
            Ok(dev_slc
                .iter()
                .map(|d| Device::from_kdev_t(*d as u32))
                .collect())
        }
    }

    /// Parse a device's table. The table value is in buf, count indicates the
    /// expected number of lines.
    /// Trims trailing white space off final entry on each line. This
    /// canonicalization makes checking identity of tables easier.
    /// Postcondition: The length of the next to last entry in any tuple is
    /// no more than 16 characters.
    fn parse_table_status(count: u32, buf: &[u8]) -> DmResult<Vec<(u64, u64, String, String)>> {
        let mut targets = Vec::new();
        if !buf.is_empty() {
            let mut next_off = 0;

            for _ in 0..count {
                let result = &buf[next_off..];
                let targ = unsafe { &*(result.as_ptr() as *const dmi::Struct_dm_target_spec) };

                let target_type = str_from_c_str(&targ.target_type)
                    .ok_or_else(|| {
                        DmError::InvalidArgument(
                            "Could not convert target type to a String".to_string(),
                        )
                    })?
                    .to_string();

                let params =
                    str_from_byte_slice(&result[size_of::<dmi::Struct_dm_target_spec>()..])
                        .ok_or_else(|| {
                            DmError::InvalidArgument(
                                "Invalid DM target parameters returned from kernel".to_string(),
                            )
                        })?
                        .to_string();

                targets.push((targ.sector_start, targ.length, target_type, params));

                next_off = targ.next as usize;
            }
        }
        Ok(targets)
    }

    /// Return the status of all targets for a device's "active"
    /// table.
    ///
    /// Returns DeviceInfo and a Vec of (sector_start, sector_length, type, params).
    ///
    /// If DM_STATUS_TABLE flag is set, returns the current table value. Otherwise
    /// returns target-specific status information.
    ///
    /// If DM_NOFLUSH is set, retrieving the target-specific status information for
    /// targets with metadata will not cause a metadata write.
    ///
    /// If DM_QUERY_INACTIVE_TABLE is set, instead return the status of the
    /// inactive table.
    ///
    /// Valid flags: DM_NOFLUSH, DM_STATUS_TABLE, DM_QUERY_INACTIVE_TABLE
    ///
    /// # Example
    ///
    /// ```no_run
    /// use devicemapper::{DM, DevId, DmFlags, DmName};
    /// let dm = DM::new().unwrap();
    ///
    /// let name = DmName::new("example-dev").expect("is valid DM name");
    /// let id = DevId::Name(name);
    /// let res = dm.table_status(&id,
    ///                           DmFlags::DM_STATUS_TABLE).unwrap();
    /// println!("{:?} {:?}", res.0.name(), res.1);
    /// ```
    #[allow(clippy::type_complexity)]
    pub fn table_status(
        &self,
        id: &DevId<'_>,
        flags: DmFlags,
    ) -> DmResult<(DeviceInfo, Vec<(u64, u64, String, String)>)> {
        let mut hdr = flags.to_ioctl_hdr(
            Some(id),
            DmFlags::DM_NOFLUSH | DmFlags::DM_STATUS_TABLE | DmFlags::DM_QUERY_INACTIVE_TABLE,
        )?;

        let (hdr_out, data_out) = self.do_ioctl(dmi::DM_TABLE_STATUS_CMD, &mut hdr, None)?;

        let status = DM::parse_table_status(hdr_out.target_count, &data_out)?;

        Ok((hdr_out, status))
    }

    /// Returns a list of each loaded target type with its name, and
    /// version broken into major, minor, and patchlevel.
    pub fn list_versions(&self) -> DmResult<Vec<(String, u32, u32, u32)>> {
        let mut hdr = DmFlags::default().to_ioctl_hdr(None, DmFlags::empty())?;

        let (_, data_out) = self.do_ioctl(dmi::DM_LIST_VERSIONS_CMD, &mut hdr, None)?;

        let mut targets = Vec::new();
        if !data_out.is_empty() {
            let mut result = &data_out[..];

            loop {
                let tver = unsafe { &*(result.as_ptr() as *const dmi::Struct_dm_target_versions) };

                let name =
                    str_from_byte_slice(&result[size_of::<dmi::Struct_dm_target_versions>()..])
                        .ok_or_else(|| {
                            DmError::InvalidArgument(
                                "Invalid DM target name returned from kernel".to_string(),
                            )
                        })?
                        .to_string();
                targets.push((name, tver.version[0], tver.version[1], tver.version[2]));

                if tver.next == 0 {
                    break;
                }

                result = &result[tver.next as usize..];
            }
        }

        Ok(targets)
    }

    /// Send a message to the device specified by id and the sector
    /// specified by sector. If sending to the whole device, set sector to
    /// None.
    pub fn target_msg(
        &self,
        id: &DevId<'_>,
        sector: Option<u64>,
        msg: &str,
    ) -> DmResult<(DeviceInfo, Option<String>)> {
        let mut hdr = DmFlags::default().to_ioctl_hdr(Some(id), DmFlags::empty())?;

        let msg_struct = dmi::Struct_dm_target_msg {
            sector: sector.unwrap_or_default(),
            ..Default::default()
        };
        let mut data_in = unsafe {
            let ptr = &msg_struct as *const dmi::Struct_dm_target_msg as *mut u8;
            slice::from_raw_parts(ptr, size_of::<dmi::Struct_dm_target_msg>()).to_vec()
        };

        data_in.extend(msg.as_bytes());
        data_in.push(b'\0');

        let (hdr_out, data_out) =
            self.do_ioctl(dmi::DM_TARGET_MSG_CMD, &mut hdr, Some(&data_in))?;

        let output = if (hdr_out.flags().bits() & DmFlags::DM_DATA_OUT.bits()) > 0 {
            Some(
                str::from_utf8(&data_out[..data_out.len() - 1])
                    .map(|res| res.to_string())
                    .map_err(|_| {
                        DmError::InvalidArgument("Could not convert output to a String".to_string())
                    })?,
            )
        } else {
            None
        };
        Ok((hdr_out, output))
    }

    /// If DM is being used to poll for events, once it indicates readiness it
    /// will continue to do so until we rearm it, which is what this method
    /// does.
    pub fn arm_poll(&self) -> DmResult<DeviceInfo> {
        let mut hdr = DmFlags::default().to_ioctl_hdr(None, DmFlags::empty())?;

        self.do_ioctl(dmi::DM_DEV_ARM_POLL_CMD, &mut hdr, None)
            .map(|(hdr, _)| hdr)
    }
}

impl AsRawFd for DM {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}
