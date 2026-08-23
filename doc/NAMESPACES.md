<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Namespaces on the command line

**Status: a gap. `--ns` is specified here and not implemented.**

## What changes at oxml 0.0.4

Until 0.0.4 the library ignored a namespace prefix in an expression
entirely: `//x:item` matched every `item` whatever its namespace, and
so did `//item`. That is a wrong answer with no error attached, and it
was fixed.

From 0.0.4:

- A prefixed name test resolves the prefix against bindings **supplied
  with the query**, not against the document's declarations.
- An **unbound prefix is a compile error**.
- An unprefixed name test matches only nodes in **no** namespace,
  which is what XPath 1.0 specifies and what every conforming engine
  does.

## Why that is a problem for this command

The library takes bindings through
`XPath::compile_with_namespaces(expr, &[("m", "urn:u")])`. This command
has no way to pass any.

So the moment the dependency is bumped to 0.0.4, `oxml query
'//m:item' f.xml` stops working and there is **nothing the user can
type to fix it**. A previously-wrong answer becomes an error with no
remedy, which is worse than either.

## The flag

```
-n, --ns <PREFIX=URI>    Bind a namespace prefix, repeatable
```

```bash
oxml query -n m=urn:example:meta -t '//m:item' catalogue.xml
oxml query -n a=urn:one -n b=urn:two -c '//a:x | //b:y' f.xml
```

Rules:

- Repeatable; later bindings for the same prefix win, so a wrapper
  script can set defaults a caller overrides.
- A malformed argument -- no `=`, empty prefix -- is a **usage** error,
  exit 2, not exit 1.
- `xml` is bound by specification and need not be given; binding it to
  anything else is a usage error.
- Applies to `query` only. `check`, `stats` and `validate` do not take
  expressions.

## Why it is not implemented

This crate links published `oxml = "0.0.3"`, which has no
`compile_with_namespaces`. Implementing the flag requires 0.0.4 to be
published, and publishing it requires
[oxml#1](https://github.com/sebastienrousseau/oxml/pull/1) to merge.

**Ordering, which matters:**

1. Merge and publish `oxml` 0.0.4.
2. Bump this crate's dependency **and add `--ns` in the same change**.

Doing step 2 in two commits leaves a build in which namespaced queries
are impossible. The dependency bump and the flag are one change.

## Until then

`namespace-uri()` selects on a namespace without naming a prefix, and
works identically in both versions:

```bash
oxml query -t "//*[namespace-uri()='urn:example' and local-name()='item']" f.xml
```

Slightly verbose, and it is not going away when `--ns` lands — it is
still the right tool when you know the URI and not the prefix.
