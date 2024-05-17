BIN := noop-client
RUN := cargo run -q
VERSION ?= $(shell cat Cargo.toml | grep version | head -n 1 | awk -F '"' '{print "v"$$2}')

.PHONY: default
default: format check test run_help run run_args run_script

.PHONY: run_help
run_help:
	# ---------------------------------------------------------------------------- #
	# run help
	$(RUN) --bin $(BIN) -- --help

.PHONY: run
run:
	# ---------------------------------------------------------------------------- #
	# run no args, except what's required
	$(RUN) --bin $(BIN) -- --endpoint=https://www.example.com/

.PHONY: run_args
run_args:
	# ---------------------------------------------------------------------------- #
	# run with args
	$(RUN) --bin $(BIN) -- --endpoint http://localhost:3000/default \
			--headers "X-Test-1:makefile1" --headers "X-Test-2:makefile2" -n 15

.PHONY: run_script
run_script:
	# ---------------------------------------------------------------------------- #
	# run with script file
	$(RUN) --bin $(BIN) -- --script=test/test_script.txt \
		--endpoint=http://localhost:3000/default --verbose

.PHONY: run_json_script
run_json_script:
	# ---------------------------------------------------------------------------- #
	# run with script file
	$(RUN) --features=json --bin $(BIN) -- --script=test/test_script.json \
		--endpoint=http://localhost:3000/default

.PHONY: run_yaml_script
run_yaml_script:
	# ---------------------------------------------------------------------------- #
	# run with script file
	$(RUN) --features=yaml --bin $(BIN) -- --script=test/test_script.yaml \
		--endpoint=http://localhost:3000/default

.PHONY: run_scripts
run_scripts: run_script run_json_script run_yaml_script

.PHONY: run_json
run_json:
	# ---------------------------------------------------------------------------- #
	# run with script file
	$(RUN) --features=json --bin $(BIN) -- --script=test/test_script.txt \
		--endpoint=http://localhost:3000/default --output=json

.PHONY: run_csv
run_csv:
	# ---------------------------------------------------------------------------- #
	# run with script file
	$(RUN) --bin $(BIN) -- --script=test/test_script.txt \
		--endpoint=http://localhost:3000/default --output=csv

.PHONY: run_outputs
run_outputs: run_csv run_json

.PHONY: run_load
run_load: clean build
	docker-compose -f ./examples/compose.yaml up -d
	# Be patient, this will take a while...
	./target/release/noop-client -f ./examples/load_script.txt -p 1024
	docker-compose -f ./examples/compose.yaml stop

.PHONY: test
test: check clean
	# cargo test
	cargo test --features=all --bin $(BIN)

.PHONY: regression
regression: test run_outputs run_scripts

.PHONY: check
check:
	# cargo check
	cargo check --features=all

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
	docker tag jmervine/noop-server:latest jmervine/noop-server:$(shell git reflog | head -n1 | cut -d' ' -f1)

.PHONY: tag
tag:
	git tag $(VERSION)
	docker tag jmervine/noop-client:latest jmervine/noop-client:$(VERSION)

.PHONY: release
release: docker tag
	git push --tags
	docker push jmervine/noop-client:latest
	docker push jmervine/noop-client:$(VERSION)
