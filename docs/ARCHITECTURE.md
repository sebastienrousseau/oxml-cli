<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Architecture

A single `main.rs`, deliberately.

## Shape

```
src/main.rs
├── run()            argument parsing and dispatch
├── read_input()     a path, or standard input
├── parse_or_report() parse, or a message with line and column
├── cmd_query()      XPath evaluation and output modes
├── cmd_validate()   XSD validation via xmlschema
├── cmd_stats()      node counts by kind, and depth
└── cmd_check()      well-formedness
```

## No argument-parsing dependency

Arguments are parsed by hand: a scan for flags, then positional
arguments. There is no `clap`.

For four commands and four flags, a dependency that pulls in a derive
macro and a builder API costs more than it saves — in compile time, in
binary size, and in the surface a security advisory can land on. The
whole of `run()` is forty lines and can be read in one sitting.

This would be the wrong call at twenty commands. It is the right one at
four, and the point to revisit is when subcommand-specific flags start
needing their own help text.

The trade-off is real: flags are accepted anywhere in the argument
list, including in positions where a more rigorous parser would reject
them, and there is no shell completion.

## Output is designed for pipes

- One result per line.
- Nothing but results on stdout; diagnostics on stderr.
- `-t` and `-c` exist so a caller does not have to post-process.

`stats` and the default `query` output are human-facing and may change.
`query -t` and `query -c` are the stable ones.

## Errors carry a position

The library returns a byte offset rather than a formatted message, and
this binary turns it into a line and column:

```
not well-formed at 1:12: at byte 11: input ended unexpectedly
```

The column counts characters, not bytes, so it is the column an editor
shows.

The byte offset appears twice because `Display for oxml::Error`
prefixes its own `at byte N`. oxml 0.0.4 adds `Display for ErrorKind`,
which renders the message alone; once this crate depends on it, the
line becomes `not well-formed at 1:12: input ended unexpectedly`. The
example asserting this output will change with it.

## What it does not do

No configuration file, no environment variables, no state. A command
whose behaviour depends on a file in your home directory behaves
differently in CI than on your laptop, and that difference surfaces as
a confusing failure rather than an obvious one.
