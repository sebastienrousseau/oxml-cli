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

The library underneath carries the heavier verification: the W3C
conformance suite, fuzzing, Miri and property tests. See
<https://github.com/sebastienrousseau/oxml/blob/main/doc/TESTING.md>.
