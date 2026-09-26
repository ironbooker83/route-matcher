# route-matcher

Most web framework routers bundle three things together: parsing route
patterns, matching a request path against them, and dispatching to a
handler. That third piece is where the testing pain lives - you end up
needing a running server, a mock request object, or the whole framework
just to check that `/users/:id/posts/*rest` matches the path you think it
does.

This crate does only the first two pieces, and does them as plain
functions: given a pattern string and a path string, what segments does
the pattern break into, and does the path match it? No `Router` struct
holding registered routes, no trait for handlers, no async, nothing that
needs a runtime to exercise. You can write a table of (pattern, path,
expected params) tuples and assert against `matches()` directly.

## Usage

```rust
use route_matcher::{matches, parse, match_path};

// One-shot: parse and match in a single call.
let params = matches("/users/:id/posts/:post_id", "/users/42/posts/7").unwrap();
assert_eq!(
    params,
    Some(vec![
        ("id".to_string(), "42".to_string()),
        ("post_id".to_string(), "7".to_string()),
    ])
);

// Parse once, match many paths - useful if you're checking a pattern
// against a large number of incoming requests.
let pattern = parse("/static/*path").unwrap();
assert_eq!(
    match_path(&pattern, "/static/css/site.css"),
    Some(vec![("path".to_string(), "css/site.css".to_string())])
);
assert_eq!(match_path(&pattern, "/other"), None);
```

## Pattern syntax

- `users` - a literal segment, matched exactly.
- `:id` - a param segment, captures exactly one path component.
- `*rest` - a wildcard segment, captures everything left (possibly empty).
  Only valid as the last segment in a pattern.

Leading and trailing slashes are ignored on both patterns and paths, so
`/users/:id`, `users/:id`, and `/users/:id/` all parse the same way.

## Route tables

`find_match` matches a path against an ordered list of `Route<T>` - a
parsed pattern paired with whatever data the caller wants attached to it
(a handler id, a closure, an enum variant). It's still a plain function
over a slice, not a struct that owns the routes or dispatches to
anything: the caller decides how to store the list and what to do with
the matched `data`.

Order is priority. `find_match` returns the first route in the slice
whose pattern matches; it does not reorder by specificity. If two
patterns could both match the same path (`/users/:id` and `/users/me`),
put the one that should win first.

```rust
use route_matcher::{find_match, parse, Route};

let routes = vec![
    Route::new(parse("/users/me").unwrap(), "current_user"),
    Route::new(parse("/users/:id").unwrap(), "get_user"),
];
let (handler, params) = find_match(&routes, "/users/me").unwrap();
assert_eq!(*handler, "current_user");
assert_eq!(params, Vec::new());
```

## Design

Every public function is pure: given the same arguments it returns the
same result, and it never reads or writes anything outside its arguments
and return value. There's no global route table and no mutable state to
reset between tests - `find_match` takes the route list as an argument
like everything else. Dispatching to whatever `data` the matched route
carries is left to the caller.

## Roadmap

- Regex-free constraints on param segments (e.g. numeric-only `:id`)
- Trailing-slash and case-sensitivity options
- Benchmarks against a large route set
- Doc examples runnable via `cargo test --doc`
