<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<h1 align="center">oxml-cli</h1>

<p align="center">
  Query, validate and inspect XML from the command line — powered by <a href="https://github.com/sebastienrousseau/oxml">oxml</a>.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/oxml-cli/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/oxml-cli/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/oxml-cli"><img src="https://img.shields.io/crates/v/oxml-cli.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/oxml-cli"><img src="https://img.shields.io/badge/docs.rs-oxml-cli-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://lib.rs/crates/oxml-cli"><img src="https://img.shields.io/badge/lib.rs-oxml-cli-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/oxml-cli"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/oxml-cli?style=for-the-badge&label=OpenSSF%20Scorecard&logo=openssf" alt="OpenSSF Scorecard" /></a>
</p>

---

## Install

```bash
cargo install --git https://github.com/sebastienrousseau/oxml-cli
```

## Usage

```text
oxml <COMMAND> [OPTIONS] [FILE]

COMMANDS:
    query <XPATH>     Evaluate an XPath expression
    validate <XSD>    Validate against an XML Schema
    stats             Summarise the document
    check             Report whether the document is well-formed
```

`FILE` defaults to standard input, so it composes with pipes.

## Examples

```bash
# Every title in the document
oxml query '//book/title' -t catalogue.xml

# Just the count
oxml query '//book' -c catalogue.xml

# A scalar expression behaves like the expression it is
oxml query 'count(//book)' catalogue.xml

# Validate, with violations on stderr so stdout stays clean
oxml validate schema.xsd catalogue.xml

# Compose
curl -s https://example.com/feed.xml | oxml query '//item/title' -t
```

## Exit status

Chosen so the tool is useful in a shell:

| Code | Meaning |
|---|---|
| `0` | Success; for `validate`, the document conforms |
| `1` | The document is invalid, or no nodes matched |
| `2` | A usage or I/O error |

`oxml query ... && echo found` therefore does what you would expect.

## Design

Argument parsing is hand-rolled rather than taken from a crate. The
surface is four subcommands and a handful of flags; a dependency would
be more code to audit than it removes, and this binary is meant to be
cheap to trust.

Diagnostics go to stderr, results to stdout, so the tool composes in a
pipeline without validation output contaminating the data.

## The oxml suite

Every member ships the **same version number**, so there is never a
compatibility table to consult. Versions advance in `0.0.1` steps along
the `0.0.x` line; `0.1.0` follows `0.0.999`.

| Crate | What it is |
|---|---|
| [`oxml`](https://github.com/sebastienrousseau/oxml) | Core — parser, tree, XPath 1.0 |
| [`oxml-cli`](https://github.com/sebastienrousseau/oxml-cli) | Command-line querying and validation |
| [`oxml-lsp`](https://github.com/sebastienrousseau/oxml-lsp) | Diagnostics for editors |
| [`oxml-mcp`](https://github.com/sebastienrousseau/oxml-mcp) | Model Context Protocol server |
| [`oxml-wasm`](https://github.com/sebastienrousseau/oxml-wasm) | WebAssembly bindings |
| [`xmlschema`](https://github.com/sebastienrousseau/xmlschema) | XSD validation |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). By participating you agree to
the [Code of Conduct](CODE_OF_CONDUCT.md).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
