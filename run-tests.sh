#!/bin/bash

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

