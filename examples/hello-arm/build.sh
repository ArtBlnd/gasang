#!/usr/bin/env sh

set -ex

rustc -Copt-level=z -Cdebuginfo=0 --target=aarch64-unknown-none hello.rs

aarch64-linux-gnu-strip -s hello

