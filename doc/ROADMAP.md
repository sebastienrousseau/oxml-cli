<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Roadmap

## Where this is

Four commands over `oxml` and `xmlschema`: `query` an XPath 1.0
expression, `validate` against an XML Schema, `stats` to summarise a
document, `check` for well-formedness. It reads standard input when
given no file, so it composes with pipes.

Argument parsing is hand-written rather than pulled from a crate. The
surface is four subcommands and a handful of flags; a dependency would
be more code to audit than it removes, and this binary is meant to be
cheap to trust. That decision is why the parser has its own fuzz
target.

The whole document is parsed into a tree before anything else happens.

## The order

**1. Streaming for the commands that do not need a tree.** `check` and
parts of `stats` answer questions that a streaming parse can answer
without holding the document, and `oxml`'s `stream` module already
reads from any `BufRead`. Today "the document is larger than memory"
is a reason not to use this tool; for those two commands it need not
be.

`query` and `validate` are not in this category: an XPath expression
can address any node from any other, and schema validation needs the
tree it is validating.

**2. Exit-status detail.** Today the statuses are 0 success, 1 invalid
or no match, 2 usage or I/O error. A script that wants to distinguish
"the schema itself was broken" from "the document did not conform"
cannot. That is a real gap for anyone using this in CI.

**3. Namespace bindings from the document.** `--ns` binds a prefix
explicitly, because from oxml 0.0.4 a prefix in an expression resolves
against bindings supplied with the query rather than against the
document. Offering an opt-in flag to adopt the document's own bindings
would remove a common annoyance without making the default ambiguous.

## What is deliberately absent

**Editing.** `xmlstarlet ed` does this. A query tool that also mutates
is two tools with one set of flags.

**Formatting and pretty-printing.** `xmllint --format` does this.

**XSLT.** `xsltproc` does this.

**XPath 2.0 or 3.1.** This is XPath 1.0 — no `matches()`, no `for`, no
sequences. The library underneath implements 1.0; the CLI cannot offer
more than it has.

**An argument-parsing dependency.** See above: the surface is small
enough that the parser is cheaper to audit than to import.

## Non-goals

Replacing `xmllint` feature for feature. This exists because a pure
Rust XML toolchain with no C dependency is useful, not because the
existing tools are bad.
