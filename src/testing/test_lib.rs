// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{
    fs::File,
    io::Read,
    os::unix::io::AsRawFd,
    panic::catch_unwind,
    path::{Path, PathBuf},
    process::Command,
    sync::Once,
};

use nix::mount::{umount2, MntFlags};
use uuid::Uuid;

use crate::{
    core::{DevId, Device, DmNameBuf, DmOptions, DmUuidBuf, DM},
    result::{DmError, DmResult, ErrorEnum},
    units::Bytes,
};

static INIT: Once = Once::new();
static mut DM_CONTEXT: Option<DM> = None;

impl DM {
    /// Returns a subset of the devices returned by list_devices(), namely
    /// the devices whose names end with DM_TEST_ID, our test device suffix.
    /// This function is useful for listing devices in tests that should not
    /// take non-test devices into account.
    pub fn list_test_devices(&self) -> Result<Vec<(DmNameBuf, Device, Option<u32>)>> {
        let mut test_devs = self.list_devices()?;
        test_devs.retain(|x| x.0.as_bytes().ends_with(DM_TEST_ID.as_bytes()));
        Ok(test_devs)
    }
}

// send IOCTL via blkgetsize64
ioctl_read!(
    /// # Safety
    ///
    /// This function is a wrapper for `libc::ioctl` and therefore is unsafe for the same reasons
    /// as other libc bindings. It accepts a file descriptor and mutable pointer so the semantics
    /// of the invoked `ioctl` command should be examined to determine the effect it will have
    /// on the resources passed to the command.
    blkgetsize64,
    0x12,
    114,
    u64
);

/// get the size of a given block device file
pub fn blkdev_size(file: &File) -> Bytes {
    let mut val: u64 = 0;

    unsafe { blkgetsize64(file.as_raw_fd(), &mut val) }.unwrap();
    Bytes(u128::from(val))
}

fn get_dm() -> &'static DM {
    unsafe {
        INIT.call_once(|| DM_CONTEXT = Some(DM::new().unwrap()));
        match DM_CONTEXT {
            Some(ref context) => context,
            _ => panic!("DM_CONTEXT.is_some()"),
        }
    }
}

/// String that is to be concatenated with test supplied name to identify
/// devices and filesystems generated by tests.
static DM_TEST_ID: &str = "_dm-rs_test_delme";

/// Generate a string with an identifying test suffix
pub fn test_string(name: &str) -> String {
    let mut namestr = String::from(name);
    namestr.push_str(DM_TEST_ID);
    namestr
}

/// Execute command while collecting stdout & stderr.
fn execute_cmd(cmd: &mut Command) -> DmResult<()> {
    match cmd.output() {
        Err(err) => Err(DmError::Dm(
            ErrorEnum::Error,
            format!("cmd: {:?}, error '{}'", cmd, err.to_string()),
        )),
        Ok(result) => {
            if result.status.success() {
                Ok(())
            } else {
                let std_out_txt = String::from_utf8_lossy(&result.stdout);
                let std_err_txt = String::from_utf8_lossy(&result.stderr);
                let err_msg = format!(
                    "cmd: {:?} stdout: {} stderr: {}",
                    cmd, std_out_txt, std_err_txt
                );
                Err(DmError::Dm(ErrorEnum::Error, err_msg))
            }
        }
    }
}

/// Generate an XFS FS, does not specify UUID as that's not supported on version in Travis
pub fn xfs_create_fs(devnode: &Path) -> DmResult<()> {
    execute_cmd(Command::new("mkfs.xfs").arg("-f").arg("-q").arg(&devnode))
}

/// Set a UUID for a XFS volume.
pub fn xfs_set_uuid(devnode: &Path, uuid: &Uuid) -> DmResult<()> {
    execute_cmd(
        Command::new("xfs_admin")
            .arg("-U")
            .arg(format!("{}", uuid))
            .arg(devnode),
    )
}

/// Wait for udev activity to be done.
pub fn udev_settle() -> DmResult<()> {
    execute_cmd(Command::new("udevadm").arg("settle"))
}

/// Generate the test name given the test supplied name.
pub fn test_name(name: &str) -> DmResult<DmNameBuf> {
    DmNameBuf::new(test_string(name))
}

/// Generate the test uuid given the test supplied name.
pub fn test_uuid(name: &str) -> DmResult<DmUuidBuf> {
    DmUuidBuf::new(test_string(name))
}

mod cleanup_errors {
    use super::DmError;

    #[derive(Debug)]
    pub enum Error {
        Ioe(std::io::Error),
        Mnt(libmount::mountinfo::ParseError),
        Nix(nix::Error),
        Msg(String),
        Chained(String, Box<Error>),
        Dm(DmError),
    }

    pub type Result<T> = std::result::Result<T, Error>;

    impl From<nix::Error> for Error {
        fn from(err: nix::Error) -> Error {
            Error::Nix(err)
        }
    }

    impl From<libmount::mountinfo::ParseError> for Error {
        fn from(err: libmount::mountinfo::ParseError) -> Error {
            Error::Mnt(err)
        }
    }

    impl From<std::io::Error> for Error {
        fn from(err: std::io::Error) -> Error {
            Error::Ioe(err)
        }
    }

    impl From<String> for Error {
        fn from(err: String) -> Error {
            Error::Msg(err)
        }
    }

    impl From<DmError> for Error {
        fn from(err: DmError) -> Error {
            Error::Dm(err)
        }
    }
}

use self::cleanup_errors::{Error, Result};

/// Attempt to remove all device mapper devices which match the test naming convention.
/// FIXME: Current implementation complicated by https://bugzilla.redhat.com/show_bug.cgi?id=1506287
fn dm_test_devices_remove() -> Result<()> {
    /// One iteration of removing devicemapper devices
    fn one_iteration() -> Result<(bool, Vec<String>)> {
        let mut progress_made = false;
        let mut remain = Vec::new();

        for n in get_dm()
            .list_test_devices()
            .map_err(|e| {
                Error::Chained(
                    "failed while listing DM devices, giving up".into(),
                    Box::new(e),
                )
            })?
            .iter()
            .map(|d| &d.0)
        {
            match get_dm().device_remove(&DevId::Name(n), DmOptions::default()) {
                Ok(_) => progress_made = true,
                Err(_) => remain.push(n.to_string()),
            }
        }
        Ok((progress_made, remain))
    }

    /// Do one iteration of removals until progress stops. Return remaining
    /// dm devices.
    fn do_while_progress() -> Result<Vec<String>> {
        let mut result = one_iteration()?;
        while result.0 {
            result = one_iteration()?;
        }
        Ok(result.1)
    }

    || -> Result<()> {
        if catch_unwind(get_dm).is_err() {
            return Err("Unable to initialize DM".to_string().into());
        }

        do_while_progress().and_then(|remain| {
            if !remain.is_empty() {
                let err_msg = format!("Some test-generated DM devices remaining: {:?}", remain);
                Err(err_msg.into())
            } else {
                Ok(())
            }
        })
    }()
    .map_err(|e| {
        Error::Chained(
            "Failed to ensure removal of all test-generated DM devices".into(),
            Box::new(e),
        )
    })
}

/// Unmount any filesystems that contain DM_TEST_ID in the mount point.
/// Return immediately on the first unmount failure.
fn dm_test_fs_unmount() -> Result<()> {
    || -> Result<()> {
        let mut mount_data = String::new();
        File::open("/proc/self/mountinfo")?.read_to_string(&mut mount_data)?;
        let parser = libmount::mountinfo::Parser::new(mount_data.as_bytes());

        for mount_point in parser
            .filter_map(|x| x.ok())
            .filter_map(|m| m.mount_point.into_owned().into_string().ok())
            .filter(|mp| mp.contains(DM_TEST_ID))
        {
            umount2(&PathBuf::from(mount_point), MntFlags::MNT_DETACH)?;
        }
        Ok(())
    }()
    .map_err(|e| {
        Error::Chained(
            "Failed to ensure all test-generated filesystems were unmounted".into(),
            Box::new(e),
        )
    })
}

/// Unmount any filesystems or devicemapper devices which contain DM_TEST_ID
/// in the path or name. Immediately return on first error.
pub fn clean_up() -> Result<()> {
    dm_test_fs_unmount().and_then(|_| dm_test_devices_remove())
}
