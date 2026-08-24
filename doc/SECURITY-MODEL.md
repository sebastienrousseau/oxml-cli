<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Security model

This is a tool you point at files from elsewhere — a downloaded feed, a
build artefact, an attachment. That is the threat model.

## It will not read a file the document asks for

An XML external entity attack declares an entity pointing at a local
path and references it, so a tool that resolves it leaks the file:

```xml
<!DOCTYPE d [<!ENTITY xxe SYSTEM "file:///etc/passwd">]>
<d>&xxe;</d>
```

`oxml` contains no code that opens a file or a socket on a document's
behalf. The entity expands to nothing. There is no flag to enable
resolution, so there is no flag to get wrong.

`xmllint` resolves external entities unless told not to. That is the
difference worth knowing when swapping one for the other in a script.

## It will not make a network request

There is no network code anywhere in the dependency tree. `oxml` reads
the file you name, or standard input, and nothing else.

## It is bounded

Entity expansion, nesting depth, attribute sizes and name lengths are
all bounded by the library's defaults, so a small hostile document
cannot turn into unbounded work.

The defaults are the library's *generous* profile, chosen so that no
real document is refused. A nine-level billion-laughs document is
refused, but it costs about 66 ms to refuse. A future release will
expose `--strict`, which refuses the same document in 25 µs.

For now: if you are running this over untrusted input in a loop, put a
timeout on it.

## Memory safety

`#![forbid(unsafe_code)]` in the library, checked in CI. No C
dependency, so no libxml2 CVE stream.

## What it does not protect you from

- **What the document says.** `oxml` reports what is in the file. If a
  value is hostile to whatever consumes it next, that is downstream.
- **A very large document.** The whole tree is built in memory.
- **Your own shell.** `oxml query -t '//cmd' f.xml | sh` is your
  decision.

Full detail, including the entity budget reasoning:
<https://github.com/sebastienrousseau/oxml/blob/main/doc/SECURITY-MODEL.md>
