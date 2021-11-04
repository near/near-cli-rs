#!/bin/sh

set -e

MSRV="1.29.0"

CMD="rustup run ${MSRV}"

rm -f Cargo.lock
$CMD cargo generate-lockfile
#$CMD cargo update --package "cc" --precise "1.0.41"
#$CMD cargo update --package "serde" --precise "1.0.98"
#$CMD cargo update --package "serde_derive" --precise "1.0.98"
#$CMD cargo update --package "byteorder" --precise "1.3.4"

$CMD cargo test --no-default-features --features all-languages
$CMD cargo test --features rand,all-languages
