// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Integration tests for mini-devicemapper.
//! These tests require root privileges and cannot safely be run in
//! parallel.

#[macro_use]
extern crate assert_matches;

mod support;
use support::{list_test_devices, test_name, test_uuid};

use devicemapper::DmIoctlCmd as dmi;
use devicemapper::{DevId, DmError, DmFlags, DM};

#[test]
/// Test that some version can be obtained.
fn sudo_test_version() {
    assert_matches!(DM::new().unwrap().version(), Ok(_));
}

#[test]
/// Test that versions for some targets can be obtained.
fn sudo_test_versions() {
    assert!(!DM::new().unwrap().list_versions().unwrap().is_empty());
}

#[test]
/// Verify that if no devices have been created the list of test devices
/// is empty.
fn sudo_test_list_devices_empty() {
    assert!(list_test_devices(&DM::new().unwrap()).unwrap().is_empty());
}

#[test]
/// Verify that if one test device has been created, it will be the only
/// test device listed.
fn sudo_test_list_devices() {
    let dm = DM::new().unwrap();
    let name = test_name("example-dev").expect("is valid DM name");
    dm.device_create(&name, None, DmFlags::default()).unwrap();

    let devices = list_test_devices(&dm).unwrap();

    assert_eq!(devices.len(), 1);

    if dm.version().unwrap().1 >= 37 {
        assert_matches!(devices.first().expect("len is 1"), (nm, _, Some(0)) if nm == &name);
    } else {
        assert_matches!(devices.first().expect("len is 1"), (nm, _, None) if nm == &name);
    }

    dm.device_remove(&DevId::Name(&name), DmFlags::default())
        .unwrap();
}

#[test]
/// Test that device creation gives a device with the expected name.
fn sudo_test_create() {
    let dm = DM::new().unwrap();
    let name = test_name("example-dev").expect("is valid DM name");
    let result = dm.device_create(&name, None, DmFlags::default()).unwrap();

    assert_eq!(result.name(), Some(&*name));
    assert_eq!(result.uuid(), None);

    dm.device_remove(&DevId::Name(&name), DmFlags::default())
        .unwrap();
}

#[test]
/// Verify that creation with a UUID results in correct name and UUID.
fn sudo_test_create_uuid() {
    let dm = DM::new().unwrap();
    let name = test_name("example-dev").expect("is valid DM name");
    let uuid = test_uuid("example-363333333333333").expect("is valid DM uuid");
    let result = dm
        .device_create(&name, Some(&uuid), DmFlags::default())
        .unwrap();

    assert_eq!(result.name(), Some(&*name));
    assert_eq!(result.uuid(), Some(&*uuid));

    dm.device_remove(&DevId::Name(&name), DmFlags::default())
        .unwrap();
}

#[test]
/// Verify that resetting uuid fails.
fn sudo_test_rename_uuid() {
    let dm = DM::new().unwrap();
    let name = test_name("example-dev").expect("is valid DM name");
    let uuid = test_uuid("example-363333333333333").expect("is valid DM uuid");
    dm.device_create(&name, Some(&uuid), DmFlags::default())
        .unwrap();

    let new_uuid = test_uuid("example-9999999999").expect("is valid DM uuid");

    assert_matches!(
        dm.device_rename(&name, &DevId::Uuid(&new_uuid)),
        Err(DmError::Ioctl(op, _, _, err)) if *err == nix::errno::Errno::EINVAL && op == dmi::DM_DEV_RENAME_CMD as u8
    );

    dm.device_remove(&DevId::Name(&name), DmFlags::default())
        .unwrap();
}

#[test]
/// Verify that resetting uuid to same uuid fails.
/// Since a device with that UUID already exists, the UUID can not be used.
fn sudo_test_rename_uuid_id() {
    let dm = DM::new().unwrap();
    let name = test_name("example-dev").expect("is valid DM name");
    let uuid = test_uuid("example-363333333333333").expect("is valid DM uuid");
    dm.device_create(&name, Some(&uuid), DmFlags::default())
        .unwrap();
    assert_matches!(
        dm.device_rename(&name, &DevId::Uuid(&uuid)),
        Err(DmError::Ioctl(op, _, _, err)) if *err == nix::errno::Errno::EBUSY && op == dmi::DM_DEV_RENAME_CMD as u8
    );

    dm.device_remove(&DevId::Name(&name), DmFlags::default())
        .unwrap();
}

#[test]
/// Verify that setting a new uuid succeeds.
/// Note that the uuid is not set in the returned dev_info.
fn sudo_test_set_uuid() {
    let dm = DM::new().unwrap();
    let name = test_name("example-dev").expect("is valid DM name");
    dm.device_create(&name, None, DmFlags::default()).unwrap();

    let uuid = test_uuid("example-363333333333333").expect("is valid DM uuid");
    let result = dm.device_rename(&name, &DevId::Uuid(&uuid)).unwrap();
    assert_eq!(result.uuid(), None);
    assert_eq!(
        dm.device_info(&DevId::Name(&name)).unwrap().uuid().unwrap(),
        &*uuid
    );
    assert_matches!(dm.device_info(&DevId::Uuid(&uuid)), Ok(_));
    dm.device_remove(&DevId::Name(&name), DmFlags::default())
        .unwrap();
}

#[test]
/// Test that device rename to same name fails.
/// Since a device with that name already exists, the name can not be used.
fn sudo_test_rename_id() {
    let dm = DM::new().unwrap();
    let name = test_name("example-dev").expect("is valid DM name");
    dm.device_create(&name, None, DmFlags::default()).unwrap();

    assert_matches!(
        dm.device_rename(&name, &DevId::Name(&name)),
        Err(DmError::Ioctl(op, _, _, err)) if *err == nix::errno::Errno::EBUSY && op == dmi::DM_DEV_RENAME_CMD as u8
    );

    dm.device_remove(&DevId::Name(&name), DmFlags::default())
        .unwrap();
}

#[test]
/// Test that device rename to different name works.
/// Verify that the only test device in the list of devices is a device
/// with the new name.
fn sudo_test_rename() {
    let dm = DM::new().unwrap();
    let name = test_name("example-dev").expect("is valid DM name");
    dm.device_create(&name, None, DmFlags::default()).unwrap();

    let new_name = test_name("example-dev-2").expect("is valid DM name");
    dm.device_rename(&name, &DevId::Name(&new_name)).unwrap();

    assert_matches!(
        dm.device_info(&DevId::Name(&name)),
        Err(DmError::Ioctl(_, _, _, err)) if *err == nix::errno::Errno::ENXIO
    );

    assert_matches!(dm.device_info(&DevId::Name(&new_name)), Ok(_));

    let devices = list_test_devices(&dm).unwrap();
    assert_eq!(devices.len(), 1);

    if dm.version().unwrap().1 >= 37 {
        assert_matches!(devices.first().expect("len is 1"), (nm, _, Some(0)) if nm == &new_name);
    } else {
        assert_matches!(devices.first().expect("len is 1"), (nm, _, None) if nm == &new_name);
    }

    let third_name = test_name("example-dev-3").expect("is valid DM name");
    dm.device_create(&third_name, None, DmFlags::default())
        .unwrap();

    assert_matches!(
        dm.device_rename(&new_name, &DevId::Name(&third_name)),
        Err(DmError::Ioctl(op, _, _, err)) if *err == nix::errno::Errno::EBUSY && op == dmi::DM_DEV_RENAME_CMD as u8
    );

    dm.device_remove(&DevId::Name(&third_name), DmFlags::default())
        .unwrap();
    dm.device_remove(&DevId::Name(&new_name), DmFlags::default())
        .unwrap();
}

#[test]
/// Renaming a device that does not exist yields an error.
fn sudo_test_rename_non_existent() {
    let new_name = test_name("new_name").expect("is valid DM name");
    assert_matches!(
        DM::new().unwrap().device_rename(
            &test_name("old_name").expect("is valid DM name"),
            &DevId::Name(&new_name)
        ),
        Err(DmError::Ioctl(op, _, _, err)) if *err == nix::errno::Errno::ENXIO && op == dmi::DM_DEV_RENAME_CMD as u8
    );
}

#[test]
/// Removing a device that does not exist yields an error.
fn sudo_test_remove_non_existent() {
    assert_matches!(
        DM::new().unwrap().device_remove(
            &DevId::Name(&test_name("junk").expect("is valid DM name")),
            DmFlags::default()
        ),
        Err(DmError::Ioctl(op, _, _, err)) if *err == nix::errno::Errno::ENXIO && op == dmi::DM_DEV_REMOVE_CMD as u8
    );
}

#[test]
/// A newly created device has no deps.
fn sudo_test_empty_deps() {
    let dm = DM::new().unwrap();
    let name = test_name("example-dev").expect("is valid DM name");
    dm.device_create(&name, None, DmFlags::default()).unwrap();

    let deps = dm
        .table_deps(&DevId::Name(&name), DmFlags::default())
        .unwrap();
    assert!(deps.is_empty());
    dm.device_remove(&DevId::Name(&name), DmFlags::default())
        .unwrap();
}

#[test]
/// Table status on a non-existent name should return an error.
fn sudo_test_table_status_non_existent() {
    assert_matches!(
        DM::new().unwrap().table_status(
            &DevId::Name(&test_name("junk").expect("is valid DM name")),
            DmFlags::default()
        ),
        Err(DmError::Ioctl(_, _, _, err)) if *err == nix::errno::Errno::ENXIO
    );
}

#[test]
/// Table status on a non-existent name with TABLE_STATUS flag errors.
fn sudo_test_table_status_non_existent_table() {
    let name = test_name("junk").expect("is valid DM name");
    assert_matches!(
        DM::new().unwrap().table_status(
            &DevId::Name(&name),
            DmFlags::DM_STATUS_TABLE
        ),
        Err(DmError::Ioctl(_, _, _, err)) if *err == nix::errno::Errno::ENXIO
    );
}

#[test]
/// The table should have an entry for a newly created device.
/// The device has no segments, so the second part of the info should
/// be empty.
/// The UUID of the returned info should be the device's UUID.
fn sudo_test_table_status() {
    let dm = DM::new().unwrap();
    let name = test_name("example-dev").expect("is valid DM name");
    let uuid = test_uuid("uuid").expect("is valid DM UUID");
    dm.device_create(&name, Some(&uuid), DmFlags::default())
        .unwrap();

    let (hdr_out, status) = dm
        .table_status(&DevId::Name(&name), DmFlags::default())
        .unwrap();
    assert!(status.is_empty());
    assert_eq!(hdr_out.uuid(), Some(&*uuid));
    dm.device_remove(&DevId::Name(&name), DmFlags::default())
        .unwrap();
}

#[test]
/// Verify that getting the status of a non-existent device specified
/// by name returns an error.
fn sudo_status_no_name() {
    let name = test_name("example_dev").expect("is valid DM name");
    assert_matches!(
        DM::new().unwrap().device_info(&DevId::Name(&name)),
        Err(DmError::Ioctl(op, _, _, err)) if *err == nix::errno::Errno::ENXIO && op == dmi::DM_DEV_STATUS_CMD as u8
    );
}

#[test]
/// Verify that creating a device with the same name twice fails.
/// Verify that creating a device with the same uuid twice fails.
fn sudo_test_double_creation() {
    let dm = DM::new().unwrap();
    let name = test_name("example-dev").expect("is valid DM name");
    let uuid = test_uuid("uuid").expect("is valid DM UUID");

    let name_alt = test_name("name-alt").expect("is valid DM name");
    let uuid_alt = test_uuid("uuid-alt").expect("is valid DM UUID");

    dm.device_create(&name, Some(&uuid), DmFlags::default())
        .unwrap();
    assert_matches!(
        dm.device_create(&name, Some(&uuid), DmFlags::default()),
        Err(DmError::Ioctl(op, _, _, err)) if *err == nix::errno::Errno::EBUSY && op == dmi::DM_DEV_CREATE_CMD as u8
    );
    assert_matches!(
        dm.device_create(&name, None, DmFlags::default()),
        Err(DmError::Ioctl(op, _, _, err)) if *err == nix::errno::Errno::EBUSY && op == dmi::DM_DEV_CREATE_CMD as u8
    );
    assert_matches!(
        dm.device_create(&name, Some(&uuid_alt), DmFlags::default()),
        Err(DmError::Ioctl(op, _, _, err)) if *err == nix::errno::Errno::EBUSY && op == dmi::DM_DEV_CREATE_CMD as u8
    );
    assert_matches!(
        dm.device_create(&name_alt, Some(&uuid), DmFlags::default()),
        Err(DmError::Ioctl(op, _, _, err)) if *err == nix::errno::Errno::EBUSY && op == dmi::DM_DEV_CREATE_CMD as u8
    );
    dm.device_remove(&DevId::Name(&name), DmFlags::default())
        .unwrap();
}
