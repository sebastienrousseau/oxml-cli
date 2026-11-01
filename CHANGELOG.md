# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.15] - 2026-11-01

### Changed

- **Upgrade to `oxml` 0.0.13 and `xmlschema` 0.0.14.** Advance exact core
  parser and XSD validator pins to `oxml = "=0.0.13"` and
  `xmlschema = "=0.0.14"`, and align package and fuzz harness metadata at
  `0.0.15`.

## [0.0.14] - 2026-11-01

### Changed

- **Ecosystem release cycle v0.0.14.** Align package and fuzz harness
  metadata at `0.0.14` across the `oxml` suite.

## [0.0.13] - 2026-10-31

### Security

- **Dual-lockfile security auditing.** Enforce `cargo audit --deny warnings`
  across both `Cargo.lock` and `fuzz/Cargo.lock` in
  `.github/workflows/security.yml`.

### Changed

- **Ecosystem release cycle v0.0.13.** Align package and fuzz harness
  metadata at `0.0.13` across the `oxml` suite.

## [0.0.12] - 2026-10-31

### Changed

- **Upgrade to `oxml` 0.0.12 and `xmlschema` 0.0.13.** Align package and
  fuzz harness metadata at `0.0.12` and refresh root and fuzz lockfiles.
- **Automated GitHub release pipeline.** Add the `release-page` workflow
  job to verify the published crates.io archive, generate CycloneDX SBOMs
  and SHA-256 checksums, and attach SLSA build provenance attestations.
- **Scorecard pull-request SARIF parity.** Normalize `pull_request`
  Scorecard SARIF output so `supply-chain/branch-protection` and
  `supply-chain/online-scm` configurations are always reported.
- **CI and release action updates.** Advance `Swatinem/rust-cache` to
  v2.9.3, `taiki-e/install-action` to v2.88.0, `github/codeql-action` to
  v4.39.1, and `rust-lang/crates-io-auth-action` to v1.0.6.

## [0.0.11] - 2026-10-17

### Changed

- Pin Scorecard execution by verified container digest and disable public
  results publication. Retain SARIF artifacts, categorize posture findings
  in GitHub code scanning, and display workflow status instead of a score.

- Permit NCSA only for libfuzzer-sys 0.4.13 in the fuzz workspace;
  retain the root license policy and check both graphs in CI.
- Require an annotated tag signed by the approved public key, with the
  exact project/version subject, and use crates.io Trusted Publishing
  instead of a stored publishing token. Registry configuration still
  requires independent confirmation before tagging.

- Align unpublished harnesses at 0.0.11 and match example coverage to
  exact source paths, with negative regression contracts in local/CI gates.
- Wait for all actual release prerequisites before compiling, propagate
  publication failures and keep the laptop helper dry-run-only.

- Apply explicit untrusted-document resource limits at the application
  boundary; query frontends also use per-call XPath evaluation budgets.

- Align the publishable package and family dependencies at 0.0.11,
  refresh root/fuzz lockfiles, and retain Rust 1.89.0 as the minimum.
- Validate closed issue/dependency changes against current sources and
  use pinned compiler, coverage, fuzz and audit tools in CI.

- **Breaking: the MSRV is now Rust 1.89.0** (was 1.86.0), one floor
  shared with the maintainer's other Rust crates.
- **The MSRV job checks its toolchain first.** It already set
  `RUSTUP_TOOLCHAIN`; it now also asserts `rustc --version` before
  building.

## [0.0.10] - 2026-10-05

### Changed

- **Upgrade to `oxml` 0.0.10 and `xmlschema` 0.0.10.** `//a[1]` now returns
  the first `a` under every parent, as XPath 1.0 specifies; write `(//a)[1]`
  for the first match in the document. `!=` and `<`/`>` on node-sets are
  existential. Expressions over 10,000 operators or 256 nesting levels are
  rejected at compile time.
- **Ecosystem release cycle v0.0.10.** Begin next iteration cycle across the
  entire lockstep suite.
- **Documentation migration.** Migrated website and rendered manual to
  `oxmllib.com/cli` (`oxmllib.github.io`) using the Lucid SSG theme.

## [0.0.9] - 2026-10-04

### Changed

- **Upgrade to `oxml` 0.0.9 and `xmlschema` 0.0.9.** Leverage generational
  slot recycling, memoized entity validation, and chunked SWAR scanning.

## [0.0.8] - 2026-08-29

### Added

- A fuzz target over the hand-rolled argument parser, run for 300
  seconds on every pull request.

- **The examples are measured against the public API.** Every `pub fn`
  must be *executed* by an example, not merely mentioned. The README
  claimed this coverage; nothing checked it.

### Security

- **`cargo audit` and `cargo deny` now actually run.** The Best
  Practices badge stated they ran against the RustSec advisory
  database. They did not -- only `oxml` had the workflow, and the
  claim had been copied here with the rest of the badge answers. This
  crate has a real dependency tree, so nothing was watching it for a
  published advisory between releases.

- Every action pinned by commit SHA, branch coverage gated, CodeQL
  added, and the Developer Certificate of Origin enforced.

## [0.0.7] - 2026-08-28

### Changed

- Built on oxml 0.0.7 and xmlschema 0.0.7. The suite ships one version
  number across all six crates.

### Added

- **A library target.** Every command moved from `src/main.rs` to
  `src/lib.rs`, and `run` now takes its output and error streams as
  parameters rather than calling `println!`. The binary supplies the
  process's own streams.

  This is what the unit tests had been missing. They could previously
  only assert `is_ok()`, because output went straight to the process's
  stdout -- a query that exited 0 having printed the wrong answer
  passed. They now assert the text.

- `benches/commands.rs`, measuring each subcommand per invocation with
  process spawn excluded. Argument handling and file I/O add roughly
  9% over the bare parse on a 200 KB file, measured as a paired
  comparison.

- An **Examples** job in CI.

### Changed

- **`stats` reports namespace nodes and a remainder.** Its lines
  stopped at comments while `nodes` counted everything, so on any
  document declaring a namespace the breakdown was two short of its
  own total and looked like an arithmetic error. Every kind is now
  printed and the sum is asserted.

### Fixed

- **The examples were not run by CI**, though README.md and
  doc/TESTING.md both said they were. Two assertions in `inspect.sh`
  had been failing against the published `oxml` for a release: the
  sample document's node total moved from 21 to 23 when namespace
  nodes entered the arena, and nothing noticed.

- `cmd_query` took a fresh lock on the process's stdout inside its
  node-set branch. Harmless while the only caller was the binary;
  found immediately once the stream became a parameter.

## [0.0.6] - 2026-08-26

### Changed

- Built on oxml 0.0.6 and xmlschema 0.0.6. The suite ships one version
  number across all six crates.

  xmlschema 0.0.6 is the substantial half of this release: its W3C
  conformance pass rate moved from 71.7% to 95.6%, and its coverage of
  the suite -- the share of tests that produce an answer meaning
  anything -- from 27.0% to 87.6%. Schemas this crate previously read
  as valid, and did not enforce, are now either enforced or reported
  as unenforceable.

## [0.0.5] - 2026-08-24

### Changed

- Built on oxml 0.0.5, which completes `XPath` 1.0: all thirteen axes
  and all 27 functions.

  **One behaviour change reaches expressions passed through this
  crate.** A function name outside the specification's library, or a
  call with the wrong number of arguments, used to compile and evaluate
  to an empty node-set. Both are now compile errors, reported with an
  offset. `starts-with("abc")` answered `true` before, because the
  absent argument read as the empty string.

  Six functions that previously answered `""` now work:
  `substring-before`, `substring-after`, `translate`, `name`, `id` and
  `lang`. So do the `following`, `preceding` and `namespace` axes.

## [0.0.4] - 2026-08-24

### Added

- `-n, --ns PREFIX=URI`, binding a namespace prefix for the query.
  oxml 0.0.4 resolves prefixes in `XPath` name tests instead of matching
  on the local part alone, so a prefixed expression needs a binding.

## [0.0.3] - 2026-08-22

### Added

- Initial release. Command-line XML querying, validation and formatting, powered by oxml
- Tracks the version line of the [`oxml`](https://github.com/sebastienrousseau/oxml)
  core, so a given version of any suite member is built and tested against
  the matching core.

[0.0.3]: https://github.com/sebastienrousseau/oxml-cli/releases/tag/v0.0.3
