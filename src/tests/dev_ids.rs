// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Tests of device ID validation.

use super::*;

// We know that the only difference between DmName and DmUuid is the
// length limit, so rather than test either of them we test a one-off
// type with a much shorter limit.  This way we only have to write the
// tests once and the assertion failures are easier to read.

const TYPE_LEN: usize = 12;

type Id = DevIdStr<TYPE_LEN>;
type IdBuf = DevIdString<TYPE_LEN>;

#[test]
/// Test for errors on an empty name.
fn test_empty_name() {
    assert_matches!(Id::new(""), Err(DmError::DeviceIdEmpty));
    assert_matches!(IdBuf::new("".into()), Err(DmError::DeviceIdEmpty));
}

#[test]
/// Test for errors on an overlong name.  The limit is TYPE_LEN - 1, not
/// TYPE_LEN, because space is reserved for a C-string terminator.
fn test_too_long_name() {
    let name = "a".repeat(TYPE_LEN);
    assert_matches!(
        Id::new(&name),
        Err(DmError::DeviceIdTooLong(limit, actual))
            if limit == TYPE_LEN - 1 && actual == TYPE_LEN,
        "expected Err(DeviceIdTooLong({}, {}))",
        TYPE_LEN - 1, TYPE_LEN
    );
    assert_matches!(
        IdBuf::new(name),
        Err(DmError::DeviceIdTooLong(limit, actual))
            if limit == TYPE_LEN - 1 && actual == TYPE_LEN,
        "expected Err(DeviceIdTooLong({}, {}))",
        TYPE_LEN - 1, TYPE_LEN
    );
}

#[test]
/// Test for the _absence_ of errors on a name that just barely fits.
fn test_max_length_name() {
    let name = "a".repeat(TYPE_LEN - 1);
    {
        let id = Id::new(&name).expect("is valid id");
        assert_eq!(id.as_bytes(), &[b'a'; TYPE_LEN - 1]);
    }
    {
        let id_buf = IdBuf::new(name).expect("is valid id");
        assert_eq!(id_buf.as_bytes(), &[b'a'; TYPE_LEN - 1]);
    }
}

#[test]
/// Test for rejection of names containing invalid characters.
fn test_name_with_bad_chars() {
    assert_matches!(Id::new("a\u{0000}b"), Err(DmError::DeviceIdHasBadChars));
    assert_matches!(
        IdBuf::new("a\u{0000}b".into()),
        Err(DmError::DeviceIdHasBadChars)
    );
    assert_matches!(Id::new("a\u{2014}b"), Err(DmError::DeviceIdHasBadChars));
    assert_matches!(
        IdBuf::new("a\u{2014}b".into()),
        Err(DmError::DeviceIdHasBadChars)
    );
}

#[test]
/// Test the concrete methods and traits of the interface.
fn test_interface() {
    let id = Id::new("id").expect("is valid id");
    let id_buf = IdBuf::new("id".into()).expect("is valid id");

    // Test as_bytes.
    assert_eq!(id.as_bytes(), b"id");
    assert_eq!(id_buf.as_bytes(), b"id");

    // Test ToOwned implementation.
    // $B.to_owned() == $O
    assert_eq!(id.to_owned(), id_buf);

    // Test Display implementation
    // X.to_string() = (*X).to_string()
    assert_eq!(id.to_string(), (*id).to_string());
    assert_eq!(id_buf.to_string(), (*id_buf).to_string());

    // Test Deref
    assert_eq!(id_buf.deref(), id);
    assert_eq!(*id_buf, *id);
}
