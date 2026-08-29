# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
