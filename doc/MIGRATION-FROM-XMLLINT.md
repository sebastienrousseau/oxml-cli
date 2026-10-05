<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Migrating from `xmllint`

## Command for command

| `xmllint` | `oxml` |
|---|---|
| `xmllint --xpath '//title' f.xml` | `oxml query '//title' f.xml` |
| `xmllint --xpath 'string(//title)' f.xml` | `oxml query -t '//title' f.xml` |
| `xmllint --xpath 'count(//book)' f.xml` | `oxml query -c '//book' f.xml` |
| `xmllint --noout f.xml` | `oxml check f.xml` |
| `xmllint --schema s.xsd --noout f.xml` | `oxml validate s.xsd f.xml` |
| `xmllint --format f.xml` | not available |
| `xmllint --c14n f.xml` | not available |
| `xmllint --xinclude f.xml` | not available |
| `xmllint --nonet f.xml` | always; there is no network code |
| `xmllint --nsclean f.xml` | not available |

## Three behavioural differences

**Exit codes.** `xmllint --xpath` exits 0 when nothing matches. `oxml
query` exits 1, so `if oxml query -c '//error' f.xml` works as a
conditional. Scripts relying on the `xmllint` behaviour need rewriting;
see [EXIT-CODES.md](EXIT-CODES.md).

**External entities.** `xmllint` resolves them unless told otherwise.
`oxml` never does, and has no option to. A document that legitimately
depends on an external entity will lose that content silently.

**Namespace prefixes in expressions.** `xmllint` resolves a prefix
against the document's declarations. `oxml` ignores prefixes entirely
in the version this crate links, and from oxml 0.0.4 resolves them
against bindings supplied with the query — which this command cannot
yet supply. Filter on `namespace-uri()`, which works in every version.
See [NAMESPACES.md](NAMESPACES.md).

## What you gain

- A single static binary from `cargo install`. No libxml2, no shared
  library, no package manager.
- Exit codes a script can branch on.
- No XXE exposure to configure.
- Memory safety guaranteed by the compiler.

## What you give up

- `--format`, `--c14n`, `--xinclude`, XSLT via `xsltproc`.
- DTD validation. `oxml validate` takes an XSD.
- Decades of libxml2 behaviour that some documents depend on.

If you use `--format` or XSLT, keep `xmllint` installed. The two
coexist happily.
