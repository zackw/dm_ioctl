use std::ops::Deref;

use crate::errors::DmError;

fn err_func(err_msg: &str) -> DmError {
    DmError::InvalidArgument(err_msg.into())
}

const TYPE_LEN: usize = 12;
str_id!(Id, IdBuf, TYPE_LEN, err_func);

#[test]
/// Test for errors on an empty name.
fn test_empty_name() {
    assert_matches!(Id::new(""), Err(DmError::InvalidArgument(_)));
    assert_matches!(IdBuf::new("".into()), Err(DmError::InvalidArgument(_)));
}

#[test]
/// Test for errors on an overlong name.
fn test_too_long_name() {
    let name = "a".repeat(TYPE_LEN + 1);
    assert_matches!(Id::new(&name), Err(DmError::InvalidArgument(_)));
    assert_matches!(IdBuf::new(name), Err(DmError::InvalidArgument(_)));
}

#[test]
/// Test the concrete methods and traits of the interface.
fn test_interface() {
    let id = Id::new("id").expect("is valid id");
    let id_buf = IdBuf::new("id".into()).expect("is valid id");

    // Test as_bytes.
    assert_eq!(id.as_bytes(), &[105u8, 100u8]);
    assert_eq!(id_buf.as_bytes(), &[105u8, 100u8]);

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
