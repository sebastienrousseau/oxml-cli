<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Assurance case

An assurance case is an argument, supported by evidence, that the
software is adequately secure for what it does. This one is
deliberately short: the strongest security claim this project makes is
about what it *cannot* do.

## What this software is

`oxml-cli` is a command-line tool for querying, validating and inspecting XML.

## What it consumes

Its inputs are XML documents and XPath expressions supplied on the command line or standard input. The threat model assumes every one of them is
hostile: a document written specifically to crash the parser, exhaust
memory, or reach something it should not.

## The claim

Untrusted input is checked at the boundaries documented in
[SECURITY.md](../SECURITY.md). This crate forbids unsafe Rust in its own
source. Those controls do not prove the absence of denial of service,
logic defects, or vulnerabilities in dependencies and runtimes.

## The argument

### Memory safety is structural, not tested for

All parsing is delegated to `oxml`, which is fuzzed and Miri-checked and never fetches external entities. This crate adds no parsing of its own.

### Resource exhaustion is bounded, not merely unlikely

The concrete source, recursion and evaluation limits, together with
caller-owned limits, are recorded in [the threat model](../SECURITY.md).
A stack overflow aborts the process, so recursion must be bounded before
entering another frame; aborting is not a mitigation. These controls do
not constitute a process-wide memory or concurrency bound.

### Correctness is measured against an external standard

The project does not grade its own homework. Where an independent
conformance suite exists it is run, its denominator is published
alongside its rate, and the result is ratcheted so an unreviewed change
in either direction fails the build.

## The evidence

- `#![forbid(unsafe_code)]`, checked by a CI job.
- 64 tests: 30 in the library over argument handling and each command, 32 driving the compiled binary end to end, and two resource-budget regressions.
- Line coverage gated at a 95% floor.
- Example scripts assert their exact output and exit codes, and run in CI.

## What this case does *not* claim

- It does not claim the absence of defects or memory corruption in
  dependencies, foreign runtimes or the compiler. Source-level unsafe-code
  prohibition is one control, not a proof about the entire process.
- It does not claim that parser budgets replace deployment limits.
  Callers must separately bound ingress, concurrent requests, wall-clock
  execution and process or WebAssembly memory.
- It does not claim independent review. This project has one
  maintainer, and no third party has audited it. That is recorded here
  rather than left to be inferred.

## Reporting a problem with this case

If you can construct an input that violates the claim above, that is a
vulnerability. See [SECURITY.md](../SECURITY.md).
