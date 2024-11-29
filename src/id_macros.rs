// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// A module to contain functionality used for generating DM ids which
// are restricted in length and format by devicemapper.

// Evaluates to an error string if the value does not match the requirements.
macro_rules! str_check {
    ($value:expr, $max_allowed_chars:expr) => {{
        let value = $value;
        let max_allowed_chars = $max_allowed_chars;
        if !value.is_ascii() {
            Some(format!("value {} has some non-ascii characters", value))
        } else {
            let num_chars = value.len();
            if num_chars == 0 {
                Some("value has zero characters".into())
            } else if num_chars > max_allowed_chars {
                Some(format!(
                    "value {} has {} chars which is greater than maximum allowed {}",
                    value, num_chars, max_allowed_chars
                ))
            } else {
                None
            }
        }
    }};
}

/// Define borrowed and owned versions of string types that guarantee
/// conformance to DM restrictions, such as maximum length.
// This implementation follows the example of Path/PathBuf as closely as
// possible.
macro_rules! str_id {
    ($B:ident, $O:ident, $MAX:ident, $err_func:ident) => {
        /// The borrowed version of the DM identifier.
        #[derive(Debug, PartialEq, Eq, Hash)]
        pub struct $B {
            inner: str,
        }

        /// The owned version of the DM identifier.
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $O {
            inner: String,
        }

        impl $B {
            /// Create a new borrowed identifier from a `&str`.
            pub fn new(value: &str) -> $crate::errors::DmResult<&$B> {
                if let Some(err_msg) = str_check!(value, $MAX - 1) {
                    return Err($err_func(&err_msg));
                }
                Ok(unsafe { &*(value as *const str as *const $B) })
            }

            /// Get the inner value as bytes
            pub fn as_bytes(&self) -> &[u8] {
                self.inner.as_bytes()
            }
        }

        impl ToOwned for $B {
            type Owned = $O;
            fn to_owned(&self) -> $O {
                $O {
                    inner: self.inner.to_owned(),
                }
            }
        }

        impl core::fmt::Display for $B {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", &self.inner)
            }
        }

        impl $O {
            /// Construct a new owned identifier.
            pub fn new(value: String) -> $crate::errors::DmResult<$O> {
                if let Some(err_msg) = str_check!(&value, $MAX - 1) {
                    return Err($err_func(&err_msg));
                }
                Ok($O { inner: value })
            }
        }

        impl AsRef<$B> for $O {
            fn as_ref(&self) -> &$B {
                self
            }
        }

        impl core::borrow::Borrow<$B> for $O {
            fn borrow(&self) -> &$B {
                self.deref()
            }
        }

        impl core::ops::Deref for $O {
            type Target = $B;
            fn deref(&self) -> &$B {
                $B::new(&self.inner).expect("inner satisfies all correctness criteria for $B::new")
            }
        }
    };
}

// This has to be at the bottom of the file or else it can't see the macros.
#[cfg(test)]
#[path = "tests/id_macros.rs"]
mod tests;
