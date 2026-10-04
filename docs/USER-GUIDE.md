<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# User guide

Recipes. For what each command does, see the [README](../README.md).

## Finding out what is in a file

Before writing a query, find out the shape:

```bash
$ oxml stats feed.xml
nodes       21
elements    7
attributes  4
text        8
comments    1
max depth   4
```

Then list the element names:

```bash
oxml query -t '//*[not(self::text())]' feed.xml | head
```

## Extracting a column

```bash
oxml query -t '//item/title' feed.xml
```

One per line, so the usual tools work:

```bash
oxml query -t '//item/title' feed.xml | sort | uniq -c | sort -rn
```

## Counting without extracting

```bash
oxml query -c '//item' feed.xml
```

Cheaper to read than piping to `wc -l`, and it does not depend on
values being newline-free.

## Testing a condition in a script

```bash
if oxml query -c '//error' build.xml > /dev/null; then
  echo "build reported errors" >&2
  exit 1
fi
```

Exit 1 means no match; exit 2 means you made a mistake. See
[EXIT-CODES.md](EXIT-CODES.md).

## Validating in CI

```yaml
- run: oxml check config.xml
- run: oxml validate schema.xsd config.xml
```

Both exit non-zero on failure, which is all a CI step needs.

## Working with documents that use namespaces

Prefixes in an expression are **not** resolved. `//x:item` and `//item`
both select every `item`. Filter on the namespace:

```bash
oxml query -t "//*[namespace-uri()='urn:example' and local-name()='item']" f.xml
```

For attributes, the same pattern needs oxml 0.0.4 or later — before
that, `namespace-uri()` returned the empty string for every attribute.

## Reading from a pipe

`FILE` defaults to standard input:

```bash
curl -s https://example.com/feed.xml | oxml query -t '//item/title'
gunzip -c archive.xml.gz | oxml stats
unzip -p report.docx word/document.xml | oxml query -c '//w:p'
```

## Checking many files

```bash
find . -name '*.xml' -print0 | xargs -0 -n1 oxml check
```

Or, stopping at the first failure:

```bash
find . -name '*.xml' -exec oxml check {} \; -quit
```

## Arithmetic in a query

XPath 1.0 has functions, and non-node results print directly:

```bash
$ oxml query 'sum(//price)' catalogue.xml
17.49
$ oxml query 'count(//book[@lang="en"])' catalogue.xml
1
$ oxml query 'string-length(//title[1])' catalogue.xml
4
```

## When a document will not parse

```bash
$ oxml check broken.xml
not well-formed at 1:12: at byte 11: input ended unexpectedly
```

The column counts characters, so it matches what an editor shows. To
see the line:

```bash
sed -n '1p' broken.xml
```

## Untrusted input

There is no `--strict` yet. If you are looping over documents from
elsewhere, bound it externally:

```bash
timeout 5s oxml check "$file" || echo "refused: $file"
```

See [SECURITY-MODEL.md](SECURITY-MODEL.md).
