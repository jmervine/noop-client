RUN := cargo run
BIN := noop-client
VERBOSE ?= false
ARGS ?= --endpoint http://localhost:3000/default --headers "X-Test-1:makefile1" \
			--headers "X-Test-2:makefile2" -n 15 --verbose=$(VERBOSE)

.PHONY: default
default: format check test run_help run run_args run_script

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
	$(RUN) --bin $(BIN) -- --script=test/test_script.txt \
		--endpoint=http://localhost:3000/default --verbose

run_load: clean build
	docker-compose -f ./examples/compose.yaml up -d
	./target/release/noop-client -f ./examples/load_script.txt -p 1024
	docker-compose -f ./examples/compose.yaml stop

.PHONY: test
test:
	# cargo test
	cargo test --bin $(BIN)

.PHONY: check
check:
	# cargo check
	cargo check

.PHONY: format
format:
	# format files
	rustfmt --emit files --edition 2018 --verbose `find ./src -type f -name "*.rs"`

.PHONY: clean
clean:
	rm -rf target/release

.PHONY: build
build: format check test target/release/noop-client

target/release/noop-client:
	cargo build --release --bin $(BIN)

.PHONY: todos
todos:
	@git grep -n TODO | grep -v Makefile | awk -F':' '{ print " - TODO["$$1":"$$2"]:"$$NF }'

.PHONY: docker
docker:
	docker build . -t jmervine/noop-client:latest
