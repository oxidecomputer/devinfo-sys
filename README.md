# devinfo-sys crate

This is an illumos system crate for interacting with 
[`libdevinfo (3LIB)`](https://illumos.org/man/3lib/libdevinfo).

## Library usage

```rust
use devinfo::get_devices;

fn main() -> Result<()> {
    let info = get_devices()?;
    // ...
}
```

## CLI usage

Show virtio (`--vendor 1af4`) virtfs (`--id 1009`) devices present on the
system.

```
root@unknown:~# ./devadm show --vendor 1af4 --id 1009
pci1af4,a
=========
property                value
--------                -----
assigned-addresses      [81002810, 0, c200, 0, 200, 82002814, 0, 80002000, 0, 2000]
class-code              [10000]
compatible              ["pci1af4,1"]
device-id               [1009]
devsel-speed            [0]
max-latency             [0]
min-grant               [0]
model                   ["S"]
pci-msix-capid-pointer  [40]
power-consumption       [1, 1]
reg                     [2800, 0, 0, 0, 0, 1002810, 0, 0, 0, 200, 2002814, 0, 0, 0, 2000]
revision-id             [0]
subsystem-id            [a]
subsystem-vendor-id     [1af4]
unit-address            ["5"]
vendor-id               [1af4]
```

## Building
```
cargo build
cargo test
```
