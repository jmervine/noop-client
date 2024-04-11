# noop-client
> **This is my first attempt at working in [Rust](https://www.rust-lang.org/). Feedback
> welcome through issues, or otherwise.**

This is a (hopefully) simple method of sending http requests (kind of like curl). Either
directly, or via a pipe delimited text file -- see (examples/test_requests.txt)[examples/test_requests.txt]).

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
  -u, --url <URL>                [default: http://localhost:3000/]
  -m, --method <METHOD>          [default: GET]
  -x, --headers <HEADERS>        [default: ]
  -i, --input <INPUT>            [default: ]
  -v, --verbose <VERBOSE>        [default: false] [possible values: true, false]
  -n, --iterations <ITERATIONS>  [default: 1]
  -h, --help                     Print help
```

### Usage - basic
```
$ cargo run --bin noop-client -- --url http://www.example.com/ -n 1
#... build output omitted ...
Received result: 1
Run took: 121.485161ms
```

### Usage - file input
```
echo "
# Comments (with '#' on as the first char) and empty lines are ignored.
# Format is '{method}|{endpoint}|{headers}
GET|http://www.example.com|User-Agent:noop-client,X-Run:run1
GET|http://www.google.com|User-Agent:noop-client,X-Run:run2
GET|http://www.heroku.com|User-Agent:noop-client,X-Run:run3

# Header can be 'key:val' or 'key=val'
GET|http://www.example.com|User-Agent=noop-client,X-Run=run4

# Empty method assumes 'GET', empty header assumes none.
|http://www.example.com|
" > script.txt

cargo run --bin noop-client -- --input=script.txt --iterations=1 --verbose=true
```