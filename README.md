<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<h1 align="center">oxml-cli</h1>

<p align="center">
  Query, validate and inspect XML from a shell — powered by
  <a href="https://github.com/sebastienrousseau/oxml">oxml</a>, with zero
  <code>unsafe</code> code.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/oxml-cli/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/oxml-cli/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/oxml-cli"><img src="https://img.shields.io/crates/v/oxml-cli.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/oxml-cli"><img src="https://img.shields.io/badge/docs.rs-oxml--cli-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://lib.rs/crates/oxml-cli"><img src="https://img.shields.io/badge/lib.rs-oxml--cli-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
</p>

---

## Contents

**Getting started**

- [Install](#install) — Cargo, from source
- [Quick Start](#quick-start) — a query in one line

**The oxml ecosystem**

- [The oxml ecosystem](#the-oxml-ecosystem) — six crates, one version

**Reference**

- [Commands](#commands) — `query`, `validate`, `stats`, `check`
- [Options](#options)
- [Exit status](#exit-status) — and why it matters in a pipeline
- [Reading from standard input](#reading-from-standard-input)
- [Migration](#migration) — from `xmllint`, `xq`, `xmlstarlet`
- [Why this approach?](#why-this-approach)
- [Capabilities in 0.0.6](#capabilities-in-006)
- [Ecosystem comparison](#ecosystem-comparison)
- [Benchmarks](#benchmarks)

**Practical**

- [Examples](#examples) — recipes that do real work
- [Configuration](#configuration)
- [When not to use oxml-cli](#when-not-to-use-oxml-cli)
- [FAQ](#faq)
- [Development](#development)
- [Security](#security)
- [Documentation](#documentation)
- [Acknowledgements](#acknowledgements)
- [License](#license)

---

## Install

```bash
cargo install oxml-cli
```

From source:

```bash
git clone https://github.com/sebastienrousseau/oxml-cli
cd oxml-cli
cargo install --path .
```

The binary is called `oxml`.

There are no C dependencies and no build toolchain requirements. If
`cargo` works, this installs.

## Quick Start

```bash
$ oxml query -t '//title' catalogue.xml
Dune
Germinal
```

```bash
$ oxml check catalogue.xml
well-formed (21 nodes)
```

```bash
$ oxml stats catalogue.xml
nodes       21
elements    7
attributes  4
text        8
comments    1
max depth   4
```

## The oxml ecosystem

| Crate | What it is |
|---|---|
| [`oxml`](https://github.com/sebastienrousseau/oxml) | The library: parser, tree, XPath 1.0 |
| [`xmlschema`](https://github.com/sebastienrousseau/xmlschema) | XSD validation |
| **`oxml-cli`** | **This crate — the command line** |
| [`oxml-wasm`](https://github.com/sebastienrousseau/oxml-wasm) | WebAssembly bindings |
| [`oxml-mcp`](https://github.com/sebastienrousseau/oxml-mcp) | Model Context Protocol server |
| [`oxml-lsp`](https://github.com/sebastienrousseau/oxml-lsp) | Language Server Protocol server |

All six ship one version number, moving in steps of 0.0.1. `oxml-cli
0.0.6` goes with `oxml 0.0.6`, and no other combination is supported.

## Commands

### `query <XPATH> [FILE]`

Evaluate an XPath 1.0 expression.

```bash
$ oxml query '//title' catalogue.xml
<title>Dune</title>
<title>Germinal</title>
```

By default each match is printed as a summary of the node. `-t` prints
the text instead, and `-c` prints only how many matched.

Non-node results — numbers, strings, booleans — are printed directly:

```bash
$ oxml query 'count(//book)' catalogue.xml
2
$ oxml query 'sum(//price)' catalogue.xml
17.49
```

### `validate <XSD> [FILE]`

Validate against an XML Schema.

```bash
$ oxml validate schema.xsd catalogue.xml
valid
```

Exit status 0 means the document conforms, 1 means it does not. The
schema comes first because the document is the argument that defaults
to standard input.

### `stats [FILE]`

Summarise the document: node counts by kind, and maximum depth.

Useful for finding out what is actually in a file before writing a
query against it, and for spotting a document that is deeper or larger
than you expected.

### `check [FILE]`

Report whether the document is well-formed.

```bash
$ oxml check broken.xml
not well-formed at 1:12: at byte 11: input ended unexpectedly
```

The position is a line and column, counted in characters, so it points
where an editor would.

## Options

| Option | Effect |
|---|---|
| `-t`, `--text` | Print matched nodes' text rather than a summary |
| `-c`, `--count` | Print only the number of matches |
| `-n`, `--ns P=URI` | Bind a namespace prefix for `query`; repeatable |
| `-h`, `--help` | Usage |
| `-V`, `--version` | Version |

`-t` and `-c` apply to `query`. Both accept the flag anywhere in the
argument list.

## Exit status

| Code | Meaning |
|---|---|
| 0 | Success; for `validate`, the document conforms |
| 1 | The document is invalid, or no nodes matched |
| 2 | A usage or I/O error |

The distinction between 1 and 2 is the point. A shell script that tests
`if oxml query ... ; then` wants to know "did anything match", and it
must not confuse that with "you passed a path that does not exist". A
tool that returns 1 for both makes `set -e` a liability.

```bash
if oxml query -c '//error' build.xml > /dev/null; then
  echo "the build reported errors"
fi
```

## Reading from standard input

`FILE` defaults to standard input, so `oxml` composes:

```bash
curl -s https://example.com/feed.xml | oxml query -t '//item/title'
```

```bash
unzip -p document.docx word/document.xml | oxml query -c '//w:p'
```

```bash
find . -name '*.xml' -exec oxml check {} \;
```

## Migration

### From `xmllint`

| `xmllint` | `oxml` |
|---|---|
| `xmllint --xpath '//title' f.xml` | `oxml query '//title' f.xml` |
| `xmllint --noout f.xml` | `oxml check f.xml` |
| `xmllint --schema s.xsd f.xml` | `oxml validate s.xsd f.xml` |
| `xmllint --format f.xml` | not available |
| `xmllint --xinclude` | not available |

`xmllint` exits 0 when `--xpath` matches nothing and prints an error to
stderr; `oxml` exits 1, which is usually what a script wants.

### From `xq` / `xmlstarlet`

| `xmlstarlet` | `oxml` |
|---|---|
| `xmlstarlet sel -t -v '//title' f.xml` | `oxml query -t '//title' f.xml` |
| `xmlstarlet val f.xml` | `oxml check f.xml` |
| `xmlstarlet val -s s.xsd f.xml` | `oxml validate s.xsd f.xml` |
| `xmlstarlet ed …` | not available — `oxml` does not write |

`xmlstarlet`'s template language is more capable and considerably
larger. `oxml query` is one expression and one output mode.

## Why this approach?

**One binary, no dependencies.** `xmllint` means libxml2, which means a
package manager, a shared library and a CVE stream. `oxml` is a static
binary produced by `cargo install`.

**Exit codes that mean something.** See [Exit status](#exit-status).

**Memory safety.** The parser underneath forbids `unsafe`, and never
fetches external entities — so `oxml` cannot be made to read
`/etc/passwd` by a document that asks it to. For a tool routinely
pointed at files from elsewhere, that matters more than throughput.

**Predictable output.** One match per line, nothing else on stdout.
Diagnostics go to stderr. It is meant to be piped.

## Capabilities in 0.0.6

- XPath 1.0 queries: ten axes, 25 functions, all four value types
- Text-only and count-only output modes
- XSD validation via `xmlschema`
- Well-formedness checking with line and column
- Document statistics by node kind, and maximum depth
- Standard input, so it composes with pipes
- Exit codes distinguishing "no match" from "usage error"
- UTF-8, UTF-16 and ISO-8859-1 input
- Namespace prefixes bound on the command line with `-n, --ns`

**Not yet:** formatting or pretty-printing, editing, XInclude, XSLT.

## Ecosystem comparison

| | `oxml` | `xmllint` | `xmlstarlet` | `xq` |
|---|---|---|---|---|
| XPath 1.0 | Yes | Yes | Yes | Yes |
| XSD validation | Yes | Yes | Yes | No |
| Editing | No | No | Yes | No |
| Formatting | No | Yes | Yes | Yes |
| XSLT | No | Yes | Yes | No |
| Single static binary | Yes | No | No | Depends |
| C dependency | None | libxml2 | libxml2 | Varies |
| Fetches external entities | **Never** | Configurable | Configurable | Varies |

## Benchmarks

None are published here, and that is deliberate: the same benchmark
binary in this suite measured 14.7 and 123.1 MB/s on one machine on one
day, at a load average above 30. A figure without its conditions is not
a measurement.

For a CLI the number that matters is usually startup plus parse, and
for documents small enough to sit in a shell pipeline both are
dominated by process creation. See
[oxml's BENCHMARKS.md](https://github.com/sebastienrousseau/oxml/blob/main/doc/BENCHMARKS.md).

## Examples

Runnable scripts live in [`examples/`](examples/), each with the sample
document it operates on. They run in CI, so they cannot rot.

| Example | What it shows |
|---|---|
| [`query-basics.sh`](examples/query-basics.sh) | Every output mode of `query` |
| [`pipeline.sh`](examples/pipeline.sh) | Standard input, exit codes, composing with other tools |
| [`inspect.sh`](examples/inspect.sh) | `stats` and `check`, including a malformed document |
| [`validate.sh`](examples/validate.sh) | XSD validation, valid and invalid |

## Configuration

There is none. No config file, no environment variables, no dotfile.

That is a decision rather than an omission: a command whose behaviour
depends on a file somewhere in your home directory behaves differently
in CI than on your laptop, and the difference surfaces as a confusing
failure. Everything `oxml` does is on the command line.

Resource limits use `oxml`'s defaults. A future release will expose
`--strict` for documents from untrusted sources.

## When not to use oxml-cli

- **You need to edit XML.** `xmlstarlet ed` does; this does not.
- **You need to format or pretty-print.** `xmllint --format` does.
- **You need XSLT.** `xsltproc`.
- **You need XPath 2.0 or 3.1.** This is 1.0, so no `matches()`, no
  `for`, no sequences.
- **The document is larger than memory.** The whole tree is built.

## FAQ

### Why is the binary called `oxml` and the crate `oxml-cli`?

Because `oxml` on crates.io is the library. The crate has to be
`oxml-cli`; the thing you type should be short.

### Why does a query that matches nothing exit 1?

So a shell script can test it. `if oxml query -c '//error' log.xml`
reads naturally and does the right thing. Exit 2 is reserved for "you
made a mistake", which is a different situation and needs a different
response.

### Can it read gzipped XML?

Not directly. Pipe it: `gunzip -c f.xml.gz | oxml query …`.

### How do I query a document with namespaces?

Bind the prefix on the command line:

```bash
oxml query -n m=urn:example:meta -t '//m:item' catalogue.xml
```

The binding lives with the *query*, not the document, so the same
command works against a document that spells the prefix differently —
only the URI has to match. `-n` is repeatable, and later bindings for
the same prefix win.

**An unbound prefix is an error**, not a silent match on the local
part:

```
$ oxml query -t '//m:item' catalogue.xml
oxml: bad XPath: unbound namespace prefix `m`; bind it with --ns m=URI
```

An **unprefixed** name test matches only nodes in no namespace, which
is what XPath 1.0 specifies. If your document declares a default
namespace, `//item` matches nothing — bind a prefix to that URI.

`namespace-uri()` still works and needs no binding, which is useful
when you know the URI and not the prefix:

```bash
oxml query -t "//*[namespace-uri()='urn:example' and local-name()='item']" f.xml
```

### Is the output stable enough to parse?

`query -t` and `query -c` are: one result per line, nothing else on
stdout. `stats` and the default `query` output are human-facing and may
change.

### Does it fetch anything over the network?

No. There is no network code anywhere in the dependency tree, and
external entities are never dereferenced — a document containing
`<!ENTITY xxe SYSTEM "file:///etc/passwd">` cannot make it read that
file. See
[oxml's SECURITY-MODEL.md](https://github.com/sebastienrousseau/oxml/blob/main/doc/SECURITY-MODEL.md).

### Why no `--format`?

Because formatting requires serialisation, and the library reads but
does not write. Adding it means adding a writer, which is a larger
piece of work than it appears — round-tripping comments, entity
references, attribute order and whitespace correctly is most of the
difficulty.

### What happens on a document that is not well-formed?

It is rejected, with a line and column. There is no recovery mode. A
parser that guesses at what a malformed document meant produces a tree
that no two tools agree on.

### Can I use it in CI?

That is the intended use.

```yaml
- run: oxml check config.xml
- run: oxml validate schema.xsd config.xml
```

Both exit non-zero on failure, which is all a CI step needs.

### Does it work on Windows?

Yes. CI builds and tests on Linux, macOS and Windows.

## Development

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all --check
./examples/run-all.sh
```

## Security

The parser underneath never dereferences external entities, so XXE is
foreclosed structurally rather than by a default. Entity expansion is
bounded. `#![forbid(unsafe_code)]`.

To report a vulnerability, see [SECURITY.md](SECURITY.md). Please do
not open a public issue.

## Documentation

- [doc/](doc/) — the decisions, in longer form
- [oxml's documentation](https://github.com/sebastienrousseau/oxml/tree/main/doc)
- [CHANGELOG.md](CHANGELOG.md)
- [CONTRIBUTING.md](CONTRIBUTING.md)

## Acknowledgements

- **[`xmllint`](https://gitlab.gnome.org/GNOME/libxml2)** — the tool
  this is measured against, and the reason the exit-code behaviour is
  what it is.
- **[`xmlstarlet`](https://xmlstar.sourceforge.net/)** — for showing
  how much a command-line XML tool can do.
- **[`jq`](https://jqlang.github.io/jq/)** — for the standard a
  pipeline-friendly query tool is held to.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
