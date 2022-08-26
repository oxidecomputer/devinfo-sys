// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Copyright 2022 Oxide Computer Company

use crate::sys::DeviceKey;
use std::io::Result;

/// Assert that we can find a CPU. Should work on any platform.
#[test]
fn find_cpu() -> Result<()> {
    let devs = crate::get_devices(false)?;
    let cpu = devs.get(&DeviceKey {
        node_name: "cpu".to_owned(),
        unit_address: Some("0".to_owned()),
    });
    assert!(cpu.is_some());

    // check that CPU has a vendor
    let cpu = cpu.unwrap();
    let vendor = cpu.props.get("vendor-id");
    assert!(vendor.is_some());

    // sanity check for non-existent property
    let not_a_thing = cpu.props.get("not-a-thing");
    assert!(not_a_thing.is_none());

    Ok(())
}
