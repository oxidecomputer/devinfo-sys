#!/bin/bash
#:
#: name = "build-and-test"
#: variety = "basic"
#: target = "helios"
#: rust_toolchain = "stable"
#: output_rules = [
#:   "/work/debug/*",
#:   "/work/release/*",
#: ]
#:

set -o errexit
set -o pipefail
set -o xtrace

cargo --version
rustc --version

banner build
ptime -m cargo build
ptime -m cargo build --release

cargo fmt -- --check
cargo clippy
cargo check

for x in debug release
do
    mkdir -p /work/$x
    cp target/$x/devadm /work/$x/devadm
done

banner test
cargo test
