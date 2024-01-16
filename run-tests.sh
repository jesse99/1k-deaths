#!/bin/bash

# killall will send a signal to the process and then return.
# When the signal is eventually delivered a message is written to stdout.
# This is annoying but adding this trap handler suppresses that.
trap 'exit 0' TERM

# bash gets weird when trying to do `command& &&`
run_state() {
	./target/debug/onek-state&
    sleep 1
}

run_logic() {
    ./target/debug/onek-logic&
    sleep 1
}

cargo build &&
run_state && run_logic &&
cargo test -p onek-state &&
cargo insta test --review -p onek-invariant
killall -q onek-logic
killall -q onek-state

