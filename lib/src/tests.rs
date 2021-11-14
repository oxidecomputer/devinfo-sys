// Copyright 2021 Oxide Computer Company
use std::io::Result;

/// Assert that we can find a CPU. Should work on any platform.
#[test]
fn find_cpu() -> Result<()> {
    let devs = crate::get_devices(false)?;
    let cpu = devs.get("cpu");
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
