# routesift

You have a list of route patterns (from a router config, an OpenAPI spec, whatever)
and a huge pile of URLs or paths - an access log, a list of crawled links, a day's
worth of request logs - and you want to know which pattern each one matches, or
whether none of them do. Doing that by eye doesn't scale past a few hundred lines,
and loading a multi-gigabyte log into a script just to grep it line by line wastes
memory you don't need to spend.

`routesift` reads one line of input at a time from stdin and matches it against a
list of named route patterns, printing the first pattern that matches (with any
captured params) or `NOMATCH`. Input is never buffered in full - a route file with
a hundred patterns run against a ten gigabyte log uses the same small, constant
amount of memory as running it against ten lines.

## Building

Stdlib only, no dependencies to fetch:

```
cargo build --release
```

## Usage

Write your routes to a file, one per line, as `<name> <pattern>`:

```
# routes.txt
health          /health
user_profile    /users/:id
user_posts      /users/:id/posts/:post_id
static_assets   /assets/*path
```

Then pipe URLs or paths at it:

```
$ printf '%s\n' \
    '/health' \
    'https://api.example.com/users/42?verbose=1' \
    '/users/42/posts/7' \
    '/assets/css/site.css' \
    '/does/not/exist' \
  | routesift routes.txt

health	/health
user_profile	id=42	https://api.example.com/users/42?verbose=1
user_posts	id=42&post_id=7	/users/42/posts/7
static_assets	path=css/site.css	/assets/css/site.css
NOMATCH	/does/not/exist
```

Output is tab-separated: the route name, then the captured params as
`key=value` pairs joined with `&` (omitted if the route has no params), then
the original input line unchanged.

Full URLs are accepted - the scheme, host, query string and fragment are
stripped before matching, so only the path is compared against your patterns.

## Pattern syntax

- `/foo/bar` - matches that exact path.
- `:name` - matches exactly one path segment, captured under `name`.
- `*name` (or bare `*`) - matches everything from that point on, including
  slashes; only valid as the last segment.

Routes are tried in the order they appear in the file and the first match
wins, same as most web framework routers.

## Summary mode

Pass `--summary` to tally match counts instead of printing every line:

```
$ routesift --summary routes.txt < access.log

health	812
user_profile	4310
user_posts	96
static_assets	1502
NOMATCH	7
```

One line per route, in the order it appears in the routes file, followed by
a `NOMATCH` count. No per-line output is printed in this mode, so it's safe
to run against the same huge logs as the default mode.

## Status

First pass. Working match logic, a streaming main loop, and a summary/count
mode; see the roadmap in the commit history for what's still missing
(regex-shaped segments, case-insensitive matching).
