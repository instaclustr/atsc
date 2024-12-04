#!/usr/bin/env bash

set -e

SCRIPT_DIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

cd $SCRIPT_DIR

cargo build --release

mkdir -p atsc
cp ../../target/release/atsc atsc

# extract the crate version from Cargo.toml
CRATE_VERSION="$(cargo metadata --format-version 1 --offline --no-deps | jq -c -M -r '.packages[] | select(.name == "atsc") | .version')"
tar -cvzf out.tar.gz atsc
mv out.tar.gz ../../atsc-linux_amd64-${CRATE_VERSION}.tar.gz

rm -rf atsc
