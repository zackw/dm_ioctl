// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Modules that support testing.

#[macro_use]
mod range_macros;

mod test_lib;
pub use test_lib::{test_name, test_uuid};

mod units;
