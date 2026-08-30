<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Testing

## The examples are the test suite

Every script in [`examples/`](../examples/) asserts its output rather
than printing it. `examples/lib.sh` provides `expect`, which compares
both the exit code and the exact output:

```bash
expect "count only" 0 '2' -- query -c '//book' "$DATA/catalogue.xml"
```

This is deliberate. A README full of example invocations goes stale the
moment behaviour changes, and nothing notices — the examples still look
right. Making them assertions means the README's own examples fail CI
when they stop being true.

```bash
./examples/run-all.sh
```

## What is covered

| Script | Covers |
|---|---|
| `query-basics.sh` | Every output mode, predicates, the attribute axis, functions returning non-node values |
| `inspect.sh` | `stats`, `check` on well-formed and malformed input, an unknown command |
| `pipeline.sh` | Standard input, exit codes in a conditional, one-result-per-line, stderr separation |
| `validate.sh` | XSD validation |

Between them every command, every flag and all three exit codes are
exercised.

## Skipped, not deleted

`query-basics.sh` skips one assertion behind `OXML_NAMESPACE_FIX`,
because selecting an attribute by namespace needs a library fix that
ships in oxml 0.0.4. It is skipped with a printed reason rather than
commented out — a commented-out test is one nobody reinstates.

## Unit and integration tests

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all --check
```

62 tests: 30 in the library over argument parsing and each command,
32 over the binary end to end.

### Why the commands take their output stream as a parameter

`run` and every `cmd_*` function write to a `&mut dyn Write` rather
than calling `println!`, so a test can read back exactly what a
command produced. Before 0.0.7 the whole CLI was `src/main.rs` and the
unit tests could only assert `is_ok()` — a query that exited 0 having
printed the wrong answer passed.

Threading the writer through found one straight away: `cmd_query`'s
node-set branch took a fresh lock on the process's stdout, shadowing
the stream it had been given. Under the binary that was invisible,
because both went to the same place.

The library underneath carries the heavier verification: the W3C
conformance suite, fuzzing, Miri and property tests. See
<https://github.com/sebastienrousseau/oxml/blob/main/doc/TESTING.md>.

## Fuzzing

```bash
cargo +nightly fuzz run run_args
```

`run_args` covers the hand-rolled argument parser — the flag parser, the `--ns PREFIX=URI` splitter and the subcommand dispatch.

10,176,342 executions have run without a crash. CI runs the target for
300 seconds on every pull request, seeded from the tracked files in
`fuzz/seeds/` — the grown corpus is build output and is not tracked,
so a run starts from the same place every time rather than from
whatever a previous run happened to discover. A crash input is
uploaded as a build artefact, because knowing only that something
broke is not much use.

## Coverage

Line coverage is gated in CI at a 95% floor. **Branch coverage is
93.8%**, gated at 80.

Branch coverage needs a nightly toolchain: `cargo llvm-cov --branch`
does not build on the version this project pins. It was recorded as
unmeasurable for a while on the strength of that one failure, which
was a conclusion drawn from a single attempt.

