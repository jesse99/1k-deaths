#!/bin/bash

# bash gets weird when trying to do `command& &&`
run_state() {
	./target/debug/onek-state&
}

cargo build &&
run_state && sleep 1 &&
cargo test -p onek-types &&
cargo insta test --review -p onek-tester
killall -q onek-state

