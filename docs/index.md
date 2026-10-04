---
title: oxml-cli — Command-Line XML Toolkit
description: Command-line XML querying, validation, and formatting, powered by oxml.
hide:
  - navigation
  - toc
---

<section class="dot-hero" markdown>

# oxml-cli

<p class="tagline">Command-line XML querying, schema validation, and formatting — powered by oxml with zero unsafe code.</p>

<div class="buttons">
  <a class="primary" href="USER-GUIDE/">User Guide →</a>
  <a href="https://github.com/sebastienrousseau/oxml-cli">GitHub</a>
  <a href="https://docs.rs/oxml-cli">docs.rs</a>
  <a href="ARCHITECTURE/">Architecture</a>
</div>

</section>

## What's inside

<div class="grid cards" markdown>

- :material-console-line:{ .lg .middle } **Instant XPath querying**

    ---

    Execute XPath 1.0 queries against any XML document or stream directly from your terminal.

    [→ User Guide](USER-GUIDE.md)

- :material-check-all:{ .lg .middle } **Schema validation**

    ---

    Validate documents against W3C XML Schema (XSD) definitions with clear error diagnostics.

    [→ Validation](USER-GUIDE.md)

- :material-format-indent-increase:{ .lg .middle } **Clean formatting**

    ---

    Pretty-print, normalize, or canonicalize XML with configurable indentation and attribute sorting.

    [→ User Guide](USER-GUIDE.md)

- :material-swap-horizontal:{ .lg .middle } **xmllint migration**

    ---

    Command-line parity and flags mapping for existing scripts using `xmllint`.

    [→ Migration Guide](MIGRATION-FROM-XMLLINT.md)

- :material-shield-check:{ .lg .middle } **Secure by design**

    ---

    Strict entity expansion budgets, XXE prevention, and zero `unsafe` memory operations.

    [→ Security Model](SECURITY-MODEL.md)

- :material-code-json:{ .lg .middle } **Deterministic exit codes**

    ---

    Stable, standardized Unix process exit codes for automated CI/CD pipeline integration.

    [→ Exit Codes](EXIT-CODES.md)

</div>

## Quick start

Install `oxml-cli`:

```bash
cargo install oxml-cli
```

Query and validate an XML file:

```bash
# Query nodes with XPath
oxml query "//book/title" books.xml

# Validate against XML Schema
oxml validate --schema schema.xsd document.xml

# Format XML
oxml format document.xml
```

## Where to next

- [**User Guide**](USER-GUIDE.md) — Complete command-line options and examples.
- [**Architecture**](ARCHITECTURE.md) — Internal design and execution model.
- [**Migration from xmllint**](MIGRATION-FROM-XMLLINT.md) — Converting `xmllint` workflows.
- [**Exit Codes**](EXIT-CODES.md) — Reference for scripts and CI gates.
- [**Security Model**](SECURITY-MODEL.md) — Resource limits and entity resolution policies.
- [**Namespaces**](NAMESPACES.md) — Prefix mapping and XML namespace handling.

## Current release

- Release notes: [GitHub Releases](https://github.com/sebastienrousseau/oxml-cli/releases)
- Crates.io: [crates.io/crates/oxml-cli](https://crates.io/crates/oxml-cli)
- Repository: [sebastienrousseau/oxml-cli](https://github.com/sebastienrousseau/oxml-cli)
