# noop-client
> **This is my first attempt at working in [Rust](https://www.rust-lang.org/). Feedback
> welcome through a comment in [Discussions > Feedback Discussion](https://github.com/jmervine/noop-client/discussions/6).**

This is a (hopefully) simple method of sending http requests (kind of like curl). Either
directly; or via a pipe delimited text file -- see [test/test_requests.txt](test/test_requests.txt).

This is still in development; but to get started; run...

```
make test run_help
```

These targets will give you some ideas on how to use it.

### Usage - help
```
$ cargo run --bin noop-client -- --help
#... build output omitted ...
This is a (hopefully) simple method of sending http requests (kind of like curl). Either directly; or via a pipe delimited text file

Usage: noop-client [OPTIONS]

Options:
  -f, --script <SCRIPT>          File path containing a list of options to be used, in place of other arguments [default: ]
  -e, --endpoint <ENDPOINT>      Target endpoint to make an http requests against [default: ]
  -m, --method <METHOD>          Method to be used when making an http requests [default: GET]
  -x, --headers <HEADERS>        Headers to be used when making an http requests [default: ]
  -n, --iterations <ITERATIONS>  Number of requests to make for each endpoint [default: 1]
  -s, --sleep <SLEEP>            Built in sleep duration (in milliseconds) to be used when making multiple requests [default: 0]
  -p, --pool-size <POOL_SIZE>    Number of parallel requests [default: 100]
  -v, --verbose                  Enable verbose output
  -D, --debug                    Enable debug output
  -E, --errors                   Enable error output for requests
  -h, --help                     Print help
  -V, --version                  Print version
```

### Usage - basic
```
$ cargo run --bin noop-client -- --endpoint=https://www.example.com/
requested=1 processed=1 success=1 fail=0 error=0 duration=172.016878ms
```

### Usage - script file
> Example uses https://github.com/jmervine/noop-server running in another window.
```
echo "
# Comments (with '#' as the first char) and empty lines are ignored.
# Format is '{iterations:-1}|{method:-GET}|{endpoint}|{headers:-}|{sleep:-0}
6|GET|http://localhost:3000/request1|User-Agent:noop-client;X-Test:run1|100

# Support all valid methods
1|POST|http://localhost:3000/request2|User-Agent:noop-client;X-Test:run2|10
1|DELETE|http://localhost:3000/request3|User-Agent:noop-client;X-Test:run3|10
1|GET|http://localhost:3000/request4|User-Agent:noop-client;X-Test:run4|10

# Empty assumes defaults; see '--help'; will error without 'endpoint' arg
||http://localhost:3000/emptyOK||

# Force an error for testing
1|GET|bad_endpoint|X-Error:true|0
" > script.txt

$ cargo run --bin noop-client -- --script=test/test_script.txt --verbose
```

output:
```
code=0 requested=11 processed=1 success=0 fail=0 error=1 duration=303.719µs
code=200 requested=11 processed=2 success=1 fail=0 error=1 duration=9.724475ms
code=200 requested=11 processed=3 success=2 fail=0 error=1 duration=14.689436ms
code=200 requested=11 processed=4 success=3 fail=0 error=1 duration=14.700042ms
code=200 requested=11 processed=5 success=4 fail=0 error=1 duration=14.978201ms
code=200 requested=11 processed=6 success=5 fail=0 error=1 duration=102.208715ms
code=200 requested=11 processed=7 success=6 fail=0 error=1 duration=103.553627ms
code=200 requested=11 processed=8 success=7 fail=0 error=1 duration=103.588955ms
code=200 requested=11 processed=9 success=8 fail=0 error=1 duration=103.63929ms
code=200 requested=11 processed=10 success=9 fail=0 error=1 duration=103.983384ms
code=200 requested=11 processed=11 success=10 fail=0 error=1 duration=104.018584ms
```
