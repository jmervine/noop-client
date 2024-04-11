RUN := cargo run
BIN := noop-client
VERBOSE ?= false
ARGS ?= --url http://localhost:3000/cli --headers "X-Test-1=makefile1" \
			--headers "X-Test-2=makefile2" -n 15000 --input examples/test_requests.txt \
			--verbose=$(VERBOSE)

.PHONY: default
default: check test run_help run run_args

.PHONY: run_help
run_help:
	# run help
	$(RUN) --bin $(BIN) -- --help

.PHONY: run 
run:
	# run no args
	$(RUN) --bin $(BIN) -- --verbose $(VERBOSE)

.PHONY: run_args 
run_args:
	# run with args
	$(RUN) --bin $(BIN) -- $(ARGS) 

.PHONY: test 
test:
	# cargo test
	cargo test

.PHONY: check
check:
	# cargo check
	cargo check

.PHONY: clean 
clean:
	rm -rf target