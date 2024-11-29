// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Tests for crate::device.

// TODO: Use proptest for more thorough validation.

use super::*;

#[test]
/// Tests of conversion from `Device` to 32-bit `kdev_t`.
fn test_device_to_kdev_t() {
    // Any 12-bit major + 20-bit minor should be representable
    // in a kdev_t.
    let dev1 = Device {
        major: 0xFED,
        minor: 0xC_BA98,
    };
    assert_eq!(dev1.to_string(), "4077:834200");
    assert_eq!(dev1.to_kdev_t(), Some(0xCBAF_ED98));

    // Going just a single bit over the limit should produce None.
    let dev2 = Device {
        major: 0x1000,
        minor: 0xC_BA98,
    };
    assert_eq!(dev2.to_string(), "4096:834200");
    assert_eq!(dev2.to_kdev_t(), None);

    let dev3 = Device {
        major: 0xFED,
        minor: 0x10_0000,
    };
    assert_eq!(dev3.to_string(), "4077:1048576");
    assert_eq!(dev3.to_kdev_t(), None);
}

#[test]
/// Tests of conversion from 64-bit extended `kdev_t` to `Device`.
fn test_device_from_kdev_t() {
    let test_devt_1 = 0x1234_5678_u64;

    let dev1 = Device::from_kdev_t(test_devt_1);
    assert_eq!(dev1.major, 0x456);
    assert_eq!(dev1.minor, 0x1_2378);
    assert_eq!(dev1.to_string(), "1110:74616");

    let dev2 = Device::from_kdev_t(0xABCD_EF12_3456_7890_u64);
    assert_eq!(dev2.major, 0xABCD_E678);
    assert_eq!(dev2.minor, 0xF123_4590);
    assert_eq!(dev2.to_string(), "2882397816:4045620624");
}
