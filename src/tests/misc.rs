// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Tests associated with the library as a whole, rather than a
//! specific submodule.

use semver::Version;

#[test]
fn lib_rs_and_cargo_toml_versions_agree() {
    let cargo_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    assert_eq!(super::VERSION, cargo_version);
}
