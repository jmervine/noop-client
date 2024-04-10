RUN := cargo run
BIN := noop-client
VERBOSE ?= false
ARGS ?= --url http://localhost:3000/cli --headers "X-Test-1=makefile1" \
			--headers "X-Test-2=makefile2" -n 5 --input examples/test_requests.txt \
			--verbose=$(VERBOSE)

default: check test run_help run run_args

run_help:
	# run help
	$(RUN) --bin $(BIN) -- --help

run:
	# run no args
	$(RUN) --bin $(BIN) -- --verbose $(VERBOSE)

run_args:
	# run with args
	$(RUN) --bin $(BIN) -- $(ARGS) 

test:
	# cargo test
	cargo test

check:
	# cargo check
	cargo check

.PHONY: test run run_help run_no_args default check