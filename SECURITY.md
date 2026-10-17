# Security Policy

## Supported versions

oxml-cli is pre-1.0. Only the most recent `0.0.x` release receives security
fixes.

## Reporting a vulnerability

Please report privately, not as a public issue:

- GitHub's [private vulnerability reporting](https://github.com/sebastienrousseau/oxml-cli/security/advisories/new)
- or email <sebastian.rousseau@gmail.com>

You should get an acknowledgement within 72 hours.

## Threat model

The CLI processes local files or stdin. Reads stop at 8 MiB; XML
parsing and XPath compilation use `Limits::strict()`. Each query uses a
fresh `EvalLimits::default()` context. Limit errors do not return partial
query results. The core library's default profile is not this CLI profile.

The parser supports bounded internal entity expansion. It does not fetch
external entities automatically. A file path supplied on the command line
is an explicit filesystem capability; do not expose it as an unrestricted
remote file-reading service.

**Memory safety.** This crate forbids unsafe Rust in its own source.
Dependencies and runtimes have separate safety obligations. This
does not rule out resource exhaustion, logic defects or protocol errors.

## What is not a vulnerability

- Rejecting a document other parsers accept, when the document is not
  well-formed. Please still report it — it is a bug, just not a
  security one.
