#!/bin/bash

# Note that `insta test` also runs unit tests.
# Also terminal atm has no unit tests.
cargo build &&
cargo test -p onek-shared &&
cargo insta test --review -p onek-backend

