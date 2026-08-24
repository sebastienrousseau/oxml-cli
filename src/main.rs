// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml. All rights reserved.

//! `oxml` — query, validate and inspect XML from the command line.
//!
//! Argument parsing is hand-rolled rather than pulled from a crate.
//! The surface is four subcommands and a handful of flags; a
//! dependency would be more code to audit than it removes, and this
//! binary is meant to be cheap to trust.

#![forbid(unsafe_code)]

use std::io::{Read as _, Write as _};
use std::process::ExitCode;

const USAGE: &str = "\
oxml — query, validate and inspect XML

USAGE:
    oxml <COMMAND> [OPTIONS] [FILE]

COMMANDS:
    query <XPATH>     Evaluate an XPath expression
    validate <XSD>    Validate against an XML Schema
    stats             Summarise the document
    check             Report whether the document is well-formed

OPTIONS:
    -t, --text        Print matched nodes' text rather than a summary
    -c, --count       Print only the number of matches
    -n, --ns P=URI    Bind a namespace prefix for `query`, repeatable
    -h, --help        Show this message
    -V, --version     Show the version

FILE defaults to standard input, so oxml composes with pipes:

    curl -s https://example.com/feed.xml | oxml query '//item/title' -t

EXIT STATUS:
    0   success, and for `validate` the document conforms
    1   the document is invalid, or no nodes matched
    2   a usage or I/O error
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("oxml: {e}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        print!("{USAGE}");
        return Ok(ExitCode::SUCCESS);
    }
    if args.iter().any(|a| a == "-V" || a == "--version") {
        println!("oxml {}", env!("CARGO_PKG_VERSION"));
        return Ok(ExitCode::SUCCESS);
    }

    let text_only = args.iter().any(|a| a == "-t" || a == "--text");
    let count_only = args.iter().any(|a| a == "-c" || a == "--count");
    let namespaces = parse_namespace_bindings(args)?;
    // `-n` takes a value, so the argument after it is not positional.
    let mut positional: Vec<&String> = Vec::new();
    let mut skip_next = false;
    for arg in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if arg == "-n" || arg == "--ns" {
            skip_next = true;
            continue;
        }
        if !arg.starts_with('-') {
            positional.push(arg);
        }
    }

    let command = positional
        .first()
        .ok_or_else(|| "no command given; try `oxml --help`".to_owned())?;

    match command.as_str() {
        "query" => {
            let expr = positional
                .get(1)
                .ok_or_else(|| "query needs an XPath expression".to_owned())?;
            let source = read_input(positional.get(2).map(|s| s.as_str()))?;
            cmd_query(&source, expr, text_only, count_only, &namespaces)
        }
        "validate" => {
            let xsd_path = positional
                .get(1)
                .ok_or_else(|| "validate needs a path to an .xsd".to_owned())?;
            let source = read_input(positional.get(2).map(|s| s.as_str()))?;
            cmd_validate(&source, xsd_path)
        }
        "stats" => {
            let source = read_input(positional.get(1).map(|s| s.as_str()))?;
            cmd_stats(&source)
        }
        "check" => {
            let source = read_input(positional.get(1).map(|s| s.as_str()))?;
            Ok(cmd_check(&source))
        }
        other => Err(format!("unknown command `{other}`; try `oxml --help`")),
    }
}

/// Read from a path, or standard input when none is given.
fn read_input(path: Option<&str>) -> Result<String, String> {
    if let Some(p) = path {
        std::fs::read_to_string(p).map_err(|e| format!("cannot read {p}: {e}"))
    } else {
        let mut buf = String::new();
        let _ = std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("cannot read stdin: {e}"))?;
        Ok(buf)
    }
}

fn parse_or_report(source: &str) -> Result<oxml::Document, String> {
    oxml::parse(source).map_err(|e| {
        let (line, col) = e.line_column(source);
        format!("{line}:{col}: {e}")
    })
}

/// Collect `-n PREFIX=URI` bindings, in order.
///
/// From oxml 0.0.4 a prefix in an expression resolves against bindings
/// supplied with the query rather than against the document, and an
/// unbound prefix is a compile error. Without a way to pass them,
/// `//m:item` would have become unanswerable from the command line --
/// a previously-wrong answer turning into an error with no remedy.
///
/// A malformed binding is a *usage* error, exit 2, not a query failure:
/// the user made a mistake rather than the document.
fn parse_namespace_bindings(
    args: &[String],
) -> Result<Vec<(String, String)>, String> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut expecting = false;
    for arg in args {
        if expecting {
            expecting = false;
            let Some((prefix, uri)) = arg.split_once('=') else {
                return Err(format!(
                    "`{arg}` is not a namespace binding; write PREFIX=URI"
                ));
            };
            if prefix.is_empty() {
                return Err("a namespace binding needs a prefix".to_owned());
            }
            if prefix == "xml" {
                return Err(
                    "`xml` is bound by the specification and may not be \
                     rebound"
                        .to_owned(),
                );
            }
            // Later bindings win, so a wrapper script can set defaults
            // a caller overrides.
            out.retain(|(p, _)| p != prefix);
            out.push((prefix.to_owned(), uri.to_owned()));
        } else if arg == "-n" || arg == "--ns" {
            expecting = true;
        }
    }
    if expecting {
        return Err("`--ns` needs a PREFIX=URI argument".to_owned());
    }
    Ok(out)
}

fn cmd_query(
    source: &str,
    expr: &str,
    text_only: bool,
    count_only: bool,
    namespaces: &[(String, String)],
) -> Result<ExitCode, String> {
    let doc = parse_or_report(source)?;
    let bindings: Vec<(&str, &str)> = namespaces
        .iter()
        .map(|(prefix, uri)| (prefix.as_str(), uri.as_str()))
        .collect();
    let xpath =
        oxml::XPath::compile_with_namespaces(expr, &bindings).map_err(|e| {
            // The library's message names a Rust function, which is no
            // help to someone at a shell. Say what they can type.
            if e.message.contains("unbound namespace prefix") {
                let prefix =
                    e.message.split('`').nth(1).unwrap_or("PREFIX").to_owned();
                format!(
                    "bad XPath: unbound namespace prefix `{prefix}`; \
                     bind it with --ns {prefix}=URI"
                )
            } else {
                format!("bad XPath: {e}")
            }
        })?;
    let value = xpath.evaluate(&doc);

    let Some(nodes) = value.nodes() else {
        // A non-node-set result — a count, a string, a boolean — is
        // printed as-is. `oxml query 'count(//x)'` should behave like
        // the expression it is.
        println!("{}", value.to_str(&doc));
        return Ok(ExitCode::SUCCESS);
    };

    if count_only {
        println!("{}", nodes.len());
        return Ok(if nodes.is_empty() {
            ExitCode::from(1)
        } else {
            ExitCode::SUCCESS
        });
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for &node in nodes {
        let line = if text_only {
            doc.text(node)
        } else {
            describe(&doc, node)
        };
        writeln!(out, "{line}").map_err(|e| e.to_string())?;
    }

    Ok(if nodes.is_empty() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

/// A one-line summary of a node, for when `--text` is not enough.
fn describe(doc: &oxml::Document, node: oxml::NodeId) -> String {
    match doc.kind(node) {
        // Resolved through `element_name` rather than destructured from
        // the variant: the element's name is interned, so the variant
        // holds a handle rather than the name itself. This accessor is
        // stable across that change.
        Some(oxml::NodeKind::Element { .. }) => {
            let Some(name) = doc.element_name(node) else {
                return String::new();
            };
            let attrs: Vec<String> = doc
                .attributes(node)
                .iter()
                .map(|a| {
                    let name =
                        doc.name(a.name).map_or("?", |n| n.local.as_str());
                    format!(" {name}=\"{}\"", a.value)
                })
                .collect();
            let text = doc.text(node);
            let preview = text.trim();
            if preview.is_empty() {
                format!("<{}{}/>", name.local, attrs.concat())
            } else {
                format!(
                    "<{}{}>{}</{}>",
                    name.local,
                    attrs.concat(),
                    truncate(preview, 60),
                    name.local
                )
            }
        }
        Some(oxml::NodeKind::Attr(a)) => {
            let name = doc.name(a.name).map_or("?", |n| n.local.as_str());
            format!("{name}=\"{}\"", a.value)
        }
        Some(oxml::NodeKind::Text(t)) => truncate(t.trim(), 80),
        Some(oxml::NodeKind::Comment(c)) => {
            format!("<!--{}-->", truncate(c.trim(), 60))
        }
        _ => String::new(),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_owned();
    }
    let head: String = s.chars().take(max).collect();
    format!("{head}…")
}

fn cmd_validate(source: &str, xsd_path: &str) -> Result<ExitCode, String> {
    let xsd = std::fs::read_to_string(xsd_path)
        .map_err(|e| format!("cannot read {xsd_path}: {e}"))?;
    let schema = xmlschema::parse_schema(&xsd)
        .map_err(|e| format!("bad schema: {e}"))?;
    let doc = parse_or_report(source)?;
    let report = xmlschema::validate(&doc, &schema);

    if report.is_valid() {
        println!("valid");
        return Ok(ExitCode::SUCCESS);
    }
    // Violations go to stderr so `oxml validate` can be used in a
    // pipeline without the diagnostics contaminating the output.
    eprintln!("{} violation(s):", report.violations.len());
    for v in &report.violations {
        eprintln!("  {} — {}", v.path, v.message);
    }
    Ok(ExitCode::from(1))
}

fn cmd_stats(source: &str) -> Result<ExitCode, String> {
    let doc = parse_or_report(source)?;
    let mut elements = 0usize;
    let mut attributes = 0usize;
    let mut text = 0usize;
    let mut comments = 0usize;
    let mut depth_max = 0usize;

    for id in doc.descendants() {
        match doc.kind(id) {
            Some(oxml::NodeKind::Element { .. }) => {
                elements += 1;
                let mut d = 0usize;
                let mut cur = Some(id);
                while let Some(n) = cur {
                    cur = doc.parent(n);
                    d += 1;
                }
                depth_max = depth_max.max(d);
            }
            Some(oxml::NodeKind::Attr(_)) => attributes += 1,
            Some(oxml::NodeKind::Text(_)) => text += 1,
            Some(oxml::NodeKind::Comment(_)) => comments += 1,
            _ => {}
        }
    }

    println!("nodes       {}", doc.len());
    println!("elements    {elements}");
    println!("attributes  {attributes}");
    println!("text        {text}");
    println!("comments    {comments}");
    println!("max depth   {depth_max}");
    Ok(ExitCode::SUCCESS)
}

/// Infallible: a parse failure is a *result* here, not an error — the
/// whole point of `check` is to report it and exit non-zero.
fn cmd_check(source: &str) -> ExitCode {
    match oxml::parse(source) {
        Ok(doc) => {
            println!("well-formed ({} nodes)", doc.len());
            ExitCode::SUCCESS
        }
        Err(e) => {
            let (line, col) = e.line_column(source);
            eprintln!("not well-formed at {line}:{col}: {e}");
            ExitCode::from(1)
        }
    }
}
