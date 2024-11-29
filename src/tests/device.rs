// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Tests for crate::device.

use super::*;

#[test]
/// Verify conversion is correct both ways
fn test_dev_t_conversion() {
    let test_devt_1: dev_t = 0xabcd_ef12_3456_7890;

    let dev1 = Device::from(test_devt_1);
    // Default glibc dev_t encoding is MMMM Mmmm mmmM MMmm. I guess if
    // we're on a platform where non-default is used, we'll fail.
    assert_eq!(dev1.major, 0xabcd_e678);
    assert_eq!(dev1.minor, 0xf123_4590);

    let test_devt_2: dev_t = dev_t::from(dev1);
    assert_eq!(test_devt_1, test_devt_2);
}

#[test]
/// Verify conversion is correct both ways
fn test_kdev_t_conversion() {
    let test_devt_1: u32 = 0x1234_5678;

    let dev1 = Device::from_kdev_t(test_devt_1);
    // Default kernel kdev_t "huge" encoding is mmmM MMmm.
    assert_eq!(dev1.major, 0x456);
    assert_eq!(dev1.minor, 0x1_2378);

    let test_devt_2: u32 = dev1.to_kdev_t().unwrap();
    assert_eq!(test_devt_1, test_devt_2);

    // a Device inexpressible as a kdev_t
    let dev2 = Device::from(0xabcd_ef12_3456_7890);
    assert_eq!(dev2.to_kdev_t(), None);
}
