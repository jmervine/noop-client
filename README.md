# noop-client
> **This is my first attempt at working in [Rust](https://www.rust-lang.org/). Feedback
> welcome through a comment in [Discussions > Feedback Discussion](https://github.com/jmervine/noop-client/discussions/6).**

This is a (hopefully) simple method of sending http requests (kind of like curl). Either
directly, or via a pipe delimited text file -- see [examples/test_requests.txt](examples/test_requests.txt).

This is still in development, but to get started, run...

```
make test run_help
```

These targets will give you some ideas on how to use it.

### Usage - help
```
$ cargo run --bin noop-client -- --help
#... build output omitted ...
Usage: noop-client [OPTIONS]

Options:
  -e, --endpoint <O_ENDPOINT>    
  -m, --method <METHOD>          [default: GET]
  -x, --headers <HEADERS>        [default: ]
  -s, --script <O_SCRIPT>        
  -v, --verbose <O_VERBOSE>      [possible values: true, false]
  -n, --iterations <ITERATIONS>  [default: 1]
  -h, --help                     Print help
```

### Usage - basic
```
$ cargo run --bin noop-client -- --endpoint http://www.example.com/
#... build output omitted ...
-------------------------
  Requests sent: 1
-------------------------
        success: 1
        failure: 0
         errors: 0
----------------------
Run took: 61.563035ms
```

### Usage - script file
> Example uses https://github.com/jmervine/noop-server running in another window.
```
echo "
# Comments (with '#' as the first char) and empty lines are ignored.
# Format is '{iterations:-1}|{method:-GET}|{endpoint}|{headers:-}
6|GET|http://localhost:3000/request1|User-Agent:noop-client,X-Test:run1

# Support all valid methods
1|POST|http://localhost:3000/request2|User-Agent:noop-client,X-Test:run2
1|DELETE|http://localhost:3000/request3|User-Agent:noop-client,X-Test:run3
1|GET|http://localhost:3000/request4|User-Agent:noop-client,X-Test:run4

# Empty assumes defaults, see '--help', will error without 'endpoint' arg
||http://localhost:3000/emptyOK|

# Force an error for testing
1|GET|bad_endpoint|X-Error:true
" > script.txt

cargo run --bin noop-client -- --script=script.txt
```

output:
```
-------------------------
  Requests sent: 11
-------------------------
        success: 10
        failure: 0
         errors: 1
----------------------
Run took: 13.617408ms
```
