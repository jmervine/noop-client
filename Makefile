RUN := cargo run
BIN := noop-client
VERBOSE ?= false
ARGS ?= --endpoint http://localhost:3000/default --headers "X-Test-1:makefile1" \
			--headers "X-Test-2=makefile2" -n 15 --verbose=$(VERBOSE)

.PHONY: default
default: check test run_help run run_args run_script

.PHONY: run_help
run_help:
	# run help
	$(RUN) --bin $(BIN) -- --help

.PHONY: run
run:
	# run no args, except what's required
	$(RUN) --bin $(BIN) -- --endpoint=https://www.example.com/

.PHONY: run_args
run_args:
	# run with args
	$(RUN) --bin $(BIN) -- $(ARGS)

run_script:
	# run with script file
	$(RUN) --bin $(BIN) -- --script=examples/test_requests.txt \
		--endpoint=http://localhost:3000/default --verbose=$(VERBOSE)

.PHONY: test
test:
	# cargo test
	cargo test --bin $(BIN)

.PHONY: functional
functional:
	make start_async_listener
	$(RUN) --bin $(BIN) -- --iterations=100000
	make stop_async_listener

.PHONY: check
check:
	# cargo check
	cargo check

.PHONY: ensure_listener
ensure_server:
	# Ensure server
	@if [ -z "$(shell which noop-server)" ]; then go install github.com/jmervine/noop-server@latest; fi

.PHONY: listener
listener: ensure_server
	# Start server with 'tee', or just start if 'tee' not found...
	#> VERBOSE=true noop-server
	@(VERBOSE=true noop-server | tee server.log) 2> /dev/null || echoVERBOSE=true noop-server

.PHONY: start_async_listener
start_async_listener: ensure_server
	#> VERBOSE=true noop-server
	@( VERBOSE=true noop-server > server.log ) &

.PHONY: stop_async_listener
stop_async_listener:
	# kill noop-server
	@kill -9 $(shell ps aux | grep noop-server | grep -v grep | awk '{ print $$2 }')

.PHONY: clean
clean:
	rm -rf target

.PHONY: format
format:
	# format files
	rustfmt --emit files --edition 2018 --verbose `find ./src -type f -name "*.rs"`
