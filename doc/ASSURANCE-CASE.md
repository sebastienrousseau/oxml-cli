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

**A hostile input can cause this software to return an error. It
cannot cause it to corrupt memory, execute code, exhaust the machine,
or reach the network or the filesystem.**

## The argument

### Memory safety is structural, not tested for

All parsing is delegated to `oxml`, which is fuzzed and Miri-checked and never fetches external entities. This crate adds no parsing of its own.

### Resource exhaustion is bounded, not merely unlikely

Depth, entity expansion and input size are bounded by explicit limits
with documented defaults. Recursion is bounded because a stack
overflow aborts the process rather than unwinding, and no caller can
catch it.

### Correctness is measured against an external standard

The project does not grade its own homework. Where an independent
conformance suite exists it is run, its denominator is published
alongside its rate, and the result is ratcheted so an unreviewed change
in either direction fails the build.

## The evidence

- `#![forbid(unsafe_code)]`, checked by a CI job.
- 61 tests: 29 in the library over argument handling and each command, 32 driving the compiled binary end to end.
- 99.29% line coverage, gated at a 95% floor.
- Example scripts assert their exact output and exit codes, and run in CI.

## What this case does *not* claim

- It does not claim the absence of defects. It claims that a defect of
  a particular class — memory corruption — is ruled out by
  construction, and that other classes are bounded and tested for.
- It does not claim the defaults are the tightest possible. They are
  chosen to accept every real document encountered; a service parsing
  untrusted XML under load should tighten them.
- It does not claim independent review. This project has one
  maintainer, and no third party has audited it. That is recorded here
  rather than left to be inferred.

## Reporting a problem with this case

If you can construct an input that violates the claim above, that is a
vulnerability. See [SECURITY.md](../SECURITY.md).
