#!/bin/bash

# killall will send a signal to the process and then return.
# When the signal is eventually delivered a message is written to stdout.
# This is annoying but adding this trap handler suppresses that.
trap 'exit 0' TERM

# bash gets weird when trying to do `command& &&`
run_backend() {
	./target/debug/onek-backend&
    sleep 1
}

run_terminal() {
    ./target/debug/onek-terminal
}

cargo build && run_backend && run_terminal
killall -q onek-backend # might be better if terminal sent an Exit message
