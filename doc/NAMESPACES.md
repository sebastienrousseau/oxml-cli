<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Namespaces on the command line

```
-n, --ns <PREFIX=URI>    Bind a namespace prefix, repeatable
```

```bash
oxml query -n m=urn:example:meta -t '//m:item' catalogue.xml
oxml query -n a=urn:one -n b=urn:two -c '//a:x | //b:y' f.xml
```

## Why bindings live with the query

From oxml 0.0.4 a prefix in an expression resolves against bindings
supplied with the query rather than against the document's
declarations. The same prefix can mean different things in the two, and
resolving against the document would make an expression's meaning
depend on which document it ran against. Only the URI has to match, so
one command works across documents that spell the prefix differently.

Before 0.0.4 the prefix was ignored entirely: `//m:item` selected every
`item` whatever its namespace. That is a wrong answer with no error
attached, which is why it changed.

## Rules

- Repeatable; later bindings for the same prefix win, so a wrapper
  script can set defaults a caller overrides.
- A malformed argument — no `=`, an empty prefix — is a **usage** error,
  exit 2, not a query failure. The user made the mistake, not the
  document.
- `xml` is bound by the specification and may not be rebound.
- Applies to `query`. `check`, `stats` and `validate` take no
  expressions.

## Unprefixed names

An unprefixed name test matches only nodes in **no** namespace. This is
XPath 1.0's classic surprise and what every conforming engine does: a
default namespace does not apply to node tests.

So against a document declaring `xmlns="urn:u"`, `//item` matches
nothing. Bind a prefix to that URI and write `//x:item`.

## Without a prefix at all

`namespace-uri()` selects on a namespace without naming one, which is
the right tool when you know the URI and not the prefix:

```bash
oxml query -t "//*[namespace-uri()='urn:example' and local-name()='item']" f.xml
```
