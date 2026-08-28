// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml. All rights reserved.

//! `oxml` — query, validate and inspect XML from the command line.
//!
//! Argument parsing is hand-rolled rather than pulled from a crate.
//! The surface is four subcommands and a handful of flags; a
//! dependency would be more code to audit than it removes, and this
//! binary is meant to be cheap to trust.

#![forbid(unsafe_code)]

use std::io::{Read as _, Write};
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

/// Run the CLI over `args`, writing to `out` and `err`.
///
/// Both streams are parameters rather than `println!` so that a test
/// or a benchmark can drive the real command path and read back what
/// it produced. Going through the binary instead measures process
/// spawn and pipe setup alongside the work, and cannot assert on
/// output without a subprocess.
///
/// # Errors
///
/// Returns the message the binary prints after `oxml: `, for usage
/// mistakes and unreadable inputs. A document that is merely invalid
/// is not an error here -- it is a non-zero [`ExitCode`].
pub fn run(
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<ExitCode, String> {
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        let _ = write!(out, "{USAGE}");
        return Ok(ExitCode::SUCCESS);
    }
    if args.iter().any(|a| a == "-V" || a == "--version") {
        let _ = writeln!(out, "oxml {}", env!("CARGO_PKG_VERSION"));
        return Ok(ExitCode::SUCCESS);
    }

    let text_only = args.iter().any(|a| a == "-t" || a == "--text");
    let count_only = args.iter().any(|a| a == "-c" || a == "--count");
    let namespaces = parse_namespace_bindings(args)?;
    let positional = positional_args(args);

    let command = positional
        .first()
        .ok_or_else(|| "no command given; try `oxml --help`".to_owned())?;

    match command.as_str() {
        "query" => {
            let expr = positional
                .get(1)
                .ok_or_else(|| "query needs an XPath expression".to_owned())?;
            let source = read_input(positional.get(2).map(|s| s.as_str()))?;
            cmd_query(&source, expr, text_only, count_only, &namespaces, out)
        }
        "validate" => {
            let xsd_path = positional
                .get(1)
                .ok_or_else(|| "validate needs a path to an .xsd".to_owned())?;
            let source = read_input(positional.get(2).map(|s| s.as_str()))?;
            cmd_validate(&source, xsd_path, out, err)
        }
        "stats" => {
            let source = read_input(positional.get(1).map(|s| s.as_str()))?;
            cmd_stats(&source, out)
        }
        "check" => {
            let source = read_input(positional.get(1).map(|s| s.as_str()))?;
            Ok(cmd_check(&source, out, err))
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
/// The positional arguments, with option values removed.
///
/// `-n` takes a value, so the argument after it is not positional.
/// Filtering on a leading `-` alone would have made `urn:example` in
/// `-n m=urn:example` look like a command.
fn positional_args(args: &[String]) -> Vec<&String> {
    let mut out: Vec<&String> = Vec::new();
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
            out.push(arg);
        }
    }
    out
}

/// Turn an `XPath` compile error into something a shell user can act on.
///
/// The library's unbound-prefix message names
/// `XPath::compile_with_namespaces`, which is no help to someone at a
/// prompt. Everything else is passed through: those messages already
/// say where in the expression the problem is.
fn explain_xpath_error(error: &oxml::XPathError) -> String {
    if error.message.contains("unbound namespace prefix") {
        let prefix = error
            .message
            .split('`')
            .nth(1)
            .unwrap_or("PREFIX")
            .to_owned();
        format!(
            "bad XPath: unbound namespace prefix `{prefix}`; \
             bind it with --ns {prefix}=URI"
        )
    } else {
        format!("bad XPath: {error}")
    }
}

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
    out: &mut dyn Write,
) -> Result<ExitCode, String> {
    let doc = parse_or_report(source)?;
    let bindings: Vec<(&str, &str)> = namespaces
        .iter()
        .map(|(prefix, uri)| (prefix.as_str(), uri.as_str()))
        .collect();
    let xpath = oxml::XPath::compile_with_namespaces(expr, &bindings)
        .map_err(|e| explain_xpath_error(&e))?;
    let value = xpath.evaluate(&doc);

    let Some(nodes) = value.nodes() else {
        // A non-node-set result — a count, a string, a boolean — is
        // printed as-is. `oxml query 'count(//x)'` should behave like
        // the expression it is.
        let _ = writeln!(out, "{}", value.to_str(&doc));
        return Ok(ExitCode::SUCCESS);
    };

    if count_only {
        let _ = writeln!(out, "{}", nodes.len());
        return Ok(if nodes.is_empty() {
            ExitCode::from(1)
        } else {
            ExitCode::SUCCESS
        });
    }

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

fn cmd_validate(
    source: &str,
    xsd_path: &str,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<ExitCode, String> {
    let xsd = std::fs::read_to_string(xsd_path)
        .map_err(|e| format!("cannot read {xsd_path}: {e}"))?;
    let schema = xmlschema::parse_schema(&xsd)
        .map_err(|e| format!("bad schema: {e}"))?;
    let doc = parse_or_report(source)?;
    let report = xmlschema::validate(&doc, &schema);

    if report.is_valid() {
        let _ = writeln!(out, "valid");
        return Ok(ExitCode::SUCCESS);
    }
    // Violations go to stderr so `oxml validate` can be used in a
    // pipeline without the diagnostics contaminating the output.
    let _ = writeln!(err, "{} violation(s):", report.violations.len());
    for v in &report.violations {
        let _ = writeln!(err, "  {} — {}", v.path, v.message);
    }
    Ok(ExitCode::from(1))
}

fn cmd_stats(source: &str, out: &mut dyn Write) -> Result<ExitCode, String> {
    let doc = parse_or_report(source)?;
    let mut elements = 0usize;
    let mut attributes = 0usize;
    let mut text = 0usize;
    let mut comments = 0usize;
    let mut namespaces = 0usize;
    let mut others = 0usize;
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
            Some(oxml::NodeKind::Namespace { .. }) => namespaces += 1,
            // The root, and processing instructions. Reported as a
            // remainder so the lines below add up to `nodes` -- see
            // the note there.
            _ => others += 1,
        }
    }

    // Every kind is printed, including the ones a reader would not
    // think to ask for. The counts previously stopped at comments,
    // which left `nodes` two larger than the lines beneath it on any
    // document with a namespace -- an apparent arithmetic error in
    // the tool, and one that stood in the sample document shipped
    // with the examples.
    let _ = writeln!(out, "nodes       {}", doc.len());
    let _ = writeln!(out, "elements    {elements}");
    let _ = writeln!(out, "attributes  {attributes}");
    let _ = writeln!(out, "text        {text}");
    let _ = writeln!(out, "comments    {comments}");
    let _ = writeln!(out, "namespaces  {namespaces}");
    let _ = writeln!(out, "other       {others}");
    let _ = writeln!(out, "max depth   {depth_max}");
    debug_assert_eq!(
        elements + attributes + text + comments + namespaces + others,
        doc.len(),
        "the breakdown must account for every node"
    );
    Ok(ExitCode::SUCCESS)
}

/// Infallible: a parse failure is a *result* here, not an error — the
/// whole point of `check` is to report it and exit non-zero.
fn cmd_check(
    source: &str,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> ExitCode {
    match oxml::parse(source) {
        Ok(doc) => {
            let _ = writeln!(out, "well-formed ({} nodes)", doc.len());
            ExitCode::SUCCESS
        }
        Err(e) => {
            let (line, col) = e.line_column(source);
            let _ = writeln!(err, "not well-formed at {line}:{col}: {e}");
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_namespace_bindings;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    /// A scratch file, removed when the test ends.
    struct Temp(std::path::PathBuf);

    impl Temp {
        fn new(name: &str, contents: &str) -> Self {
            let path = std::env::temp_dir().join(name);
            std::fs::write(&path, contents).expect("write scratch file");
            Self(path)
        }
        fn path(&self) -> &str {
            self.0.to_str().expect("utf-8 path")
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    /// Run a query and return what it printed alongside its code.
    ///
    /// Before the library target existed these assertions could only
    /// reach `is_ok()`: the output went straight to the process's
    /// stdout, so a query that exited 0 having printed the wrong
    /// answer passed.
    fn query(
        doc: &str,
        expr: &str,
        text_only: bool,
        count_only: bool,
        ns: &[(String, String)],
    ) -> Result<(String, String), String> {
        let mut out = Vec::new();
        let code =
            super::cmd_query(doc, expr, text_only, count_only, ns, &mut out)?;
        Ok((String::from_utf8(out).expect("utf-8"), format!("{code:?}")))
    }

    #[test]
    fn cmd_query_reports_matches_and_their_absence() {
        let doc = r"<r><t>one</t><t>two</t></r>";
        let success = format!("{:?}", std::process::ExitCode::SUCCESS);

        let (text, code) = query(doc, "//t", true, false, &[]).expect("valid");
        assert_eq!(text, "one\ntwo\n");
        assert_eq!(code, success);

        let (text, code) = query(doc, "//t", false, true, &[]).expect("valid");
        assert_eq!(text, "2\n");
        assert_eq!(code, success);

        // A non-node-set result prints as itself.
        let (text, code) =
            query(doc, "count(//t)", false, false, &[]).expect("valid");
        assert_eq!(text, "2\n");
        assert_eq!(code, success);

        // Exit 1 when nothing matched, so `if oxml query …` reads as a
        // question about the document -- and nothing is printed, so a
        // caller piping the output sees an empty result, not a blank
        // line.
        let (text, code) =
            query(doc, "//missing", true, false, &[]).expect("valid");
        assert_eq!(text, "");
        assert_eq!(code, format!("{:?}", std::process::ExitCode::from(1)));
    }

    #[test]
    fn cmd_query_binds_namespaces_and_refuses_unbound_ones() {
        let doc = r#"<r xmlns:m="urn:u"><m:t>ns</m:t></r>"#;
        let bound = [(String::from("m"), String::from("urn:u"))];
        let (text, _) =
            query(doc, "//m:t", true, false, &bound).expect("bound");
        assert_eq!(text, "ns\n");

        let err = query(doc, "//m:t", true, false, &[]).expect_err("unbound");
        assert!(err.contains("--ns"), "{err}");
    }

    #[test]
    fn cmd_validate_reads_a_schema_from_disk() {
        let schema = Temp::new(
            "oxml-cli-test-schema.xsd",
            r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="r"/>
</xs:schema>"#,
        );
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert!(
            super::cmd_validate("<r/>", schema.path(), &mut out, &mut err)
                .is_ok()
        );
        // A schema that is not there is a usage error, not a document
        // that failed to validate.
        assert!(
            super::cmd_validate("<r/>", "/nonexistent.xsd", &mut out, &mut err)
                .is_err()
        );
    }

    /// Drive `run` and hand back what each stream received.
    fn drive(argv: &[String]) -> Result<(String, String, String), String> {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = super::run(argv, &mut out, &mut err)?;
        Ok((
            String::from_utf8(out).expect("utf-8"),
            String::from_utf8(err).expect("utf-8"),
            format!("{code:?}"),
        ))
    }

    #[test]
    fn run_dispatches_each_command() {
        let doc = Temp::new("oxml-cli-test-doc.xml", "<r><t>x</t></r>");
        for command in [
            vec!["check", doc.path()],
            vec!["stats", doc.path()],
            vec!["query", "//t", doc.path()],
            vec!["query", "-t", "//t", doc.path()],
        ] {
            let owned = args(&command);
            let (out, _, _) = drive(&owned).expect("{command:?}");
            assert!(!out.is_empty(), "{command:?} printed nothing");
        }
    }

    #[test]
    fn run_answers_help_and_version_without_reading_anything() {
        // These must not touch stdin: `oxml --help` in a terminal with
        // no input would otherwise hang.
        for flags in [vec!["--help"], vec!["-h"], vec!["--version"], vec!["-V"]]
        {
            let owned = args(&flags);
            let (out, _, _) = drive(&owned).expect("{flags:?}");
            assert!(!out.is_empty(), "{flags:?} printed nothing");
        }
    }

    #[test]
    fn run_rejects_a_command_it_does_not_have() {
        let owned = args(&["frobnicate"]);
        let err = drive(&owned).expect_err("unknown");
        assert!(err.contains("unknown command"), "{err}");

        // And a command that needs an argument it was not given.
        let owned = args(&["query"]);
        assert!(drive(&owned).is_err());
        let owned = args(&["validate"]);
        assert!(drive(&owned).is_err());
        let owned: Vec<String> = Vec::new();
        let (usage, _, _) = drive(&owned).expect("no arguments prints usage");
        assert!(usage.contains("USAGE") || usage.contains("oxml"), "{usage}");
    }

    #[test]
    fn truncate_keeps_short_strings_whole() {
        assert_eq!(super::truncate("short", 10), "short");
        assert_eq!(super::truncate("exactly10!", 10), "exactly10!");
    }

    #[test]
    fn truncate_counts_characters_not_bytes() {
        // Cutting at a byte offset would split a multi-byte character
        // and panic. Five accented characters are ten bytes.
        assert_eq!(super::truncate("ééééé", 3), "ééé…");
        assert_eq!(super::truncate("ééééé", 5), "ééééé");
    }

    #[test]
    fn describe_renders_each_node_kind() {
        let doc =
            super::parse_or_report(r#"<r a="1"><!--note--><?pi go?>text</r>"#)
                .expect("well-formed");
        let rendered: Vec<String> = doc
            .descendants()
            .map(|id| super::describe(&doc, id))
            .collect();
        let all = rendered.join("\n");
        assert!(
            all.contains("<r a=\"1\">"),
            "element with attributes: {all}"
        );
        assert!(all.contains("note"), "comment: {all}");
        // A processing instruction falls to the catch-all arm and
        // renders as nothing. Worth pinning: a summary that silently
        // omits a node kind is easy to grow and hard to notice.
        assert!(
            rendered.iter().any(String::is_empty),
            "a processing instruction renders empty: {rendered:?}"
        );
        assert!(all.contains("text"), "text: {all}");
    }

    #[test]
    fn parse_or_report_explains_where_the_document_broke() {
        let err = super::parse_or_report("<a></b>").expect_err("mismatched");
        // A line and column, because a byte offset sends the reader
        // counting.
        assert!(err.contains("1:4"), "{err}");
    }

    #[test]
    fn cmd_check_separates_a_bad_document_from_a_good_one() {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(
            format!("{:?}", super::cmd_check("<a/>", &mut out, &mut err)),
            format!("{:?}", std::process::ExitCode::SUCCESS)
        );
        assert!(
            String::from_utf8(out)
                .expect("utf-8")
                .contains("well-formed"),
            "a good document says so"
        );

        // Exit 1: the document has a problem. Exit 2 is reserved for
        // the caller having made a mistake.
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(
            format!("{:?}", super::cmd_check("<a>", &mut out, &mut err)),
            format!("{:?}", std::process::ExitCode::from(1))
        );
        // The diagnosis goes to stderr, so `oxml check x.xml > out`
        // still shows the caller what was wrong.
        assert!(out.is_empty(), "a failure printed to stdout");
        assert!(
            String::from_utf8(err)
                .expect("utf-8")
                .contains("not well-formed"),
            "a bad document says why"
        );
    }

    #[test]
    fn cmd_stats_counts_a_document_without_touching_the_filesystem() {
        let mut out = Vec::new();
        assert!(
            super::cmd_stats("<r><a/><b>x</b><!--c--></r>", &mut out).is_ok()
        );
        let text = String::from_utf8(out).expect("utf-8");
        assert!(text.contains("elements    3"), "{text}");
        assert!(text.contains("comments    1"), "{text}");
        // A malformed document is reported rather than counted.
        let mut out = Vec::new();
        assert!(super::cmd_stats("<r>", &mut out).is_err());
    }

    #[test]
    fn cmd_stats_accounts_for_every_node() {
        // The kind lines must sum to `nodes`. They did not before
        // 0.0.7: namespace nodes and the document root were counted
        // in the total and printed nowhere, so `stats` on any
        // namespaced document looked as though it could not add up.
        let doc = r#"<r xmlns:m="urn:u"><m:a k="v">x</m:a><!--c--></r>"#;
        let mut out = Vec::new();
        let _ = super::cmd_stats(doc, &mut out).expect("well-formed");
        let text = String::from_utf8(out).expect("utf-8");

        let value = |label: &str| -> usize {
            text.lines()
                .find(|l| l.starts_with(label))
                .and_then(|l| l.split_whitespace().last())
                .and_then(|n| n.parse().ok())
                .unwrap_or_else(|| panic!("no `{label}` line in {text}"))
        };
        let total = value("nodes");
        let parts = [
            "elements",
            "attributes",
            "text",
            "comments",
            "namespaces",
            "other",
        ]
        .iter()
        .map(|l| value(l))
        .sum::<usize>();
        assert_eq!(parts, total, "the breakdown does not add up:\n{text}");
        // And the namespaces are really counted: `m`, plus the
        // implicit `xml` binding every document carries.
        assert_eq!(value("namespaces"), 2, "{text}");
    }

    #[test]
    fn an_option_value_is_not_a_positional_argument() {
        // Filtering on a leading `-` alone would have made
        // `urn:example` look like a command, because a URI does not
        // begin with a dash.
        let owned = args(&["query", "-n", "m=urn:example", "//m:x", "f.xml"]);
        let got = super::positional_args(&owned);
        let got: Vec<&str> = got.iter().map(|s| s.as_str()).collect();
        assert_eq!(got, ["query", "//m:x", "f.xml"]);
    }

    #[test]
    fn ordinary_flags_are_still_dropped() {
        let owned = args(&["query", "-t", "//x", "-c", "f.xml"]);
        let got = super::positional_args(&owned);
        let got: Vec<&str> = got.iter().map(|s| s.as_str()).collect();
        assert_eq!(got, ["query", "//x", "f.xml"]);
    }

    #[test]
    fn a_trailing_option_consumes_nothing_that_is_not_there() {
        let owned = args(&["query", "//x", "-n"]);
        let got = super::positional_args(&owned);
        let got: Vec<&str> = got.iter().map(|s| s.as_str()).collect();
        assert_eq!(got, ["query", "//x"]);
    }

    #[test]
    fn the_long_option_form_also_takes_a_value() {
        let owned = args(&["query", "--ns", "m=urn:u", "//m:x"]);
        let got = super::positional_args(&owned);
        let got: Vec<&str> = got.iter().map(|s| s.as_str()).collect();
        assert_eq!(got, ["query", "//m:x"]);
    }

    #[test]
    fn an_unbound_prefix_is_explained_in_terms_of_the_flag() {
        // The library says "bind it with
        // XPath::compile_with_namespaces", which is a Rust function.
        // Someone at a shell needs `--ns`.
        let err = oxml::XPath::compile("//m:item").expect_err("unbound");
        let said = super::explain_xpath_error(&err);
        assert!(said.contains("--ns m=URI"), "{said}");
        assert!(!said.contains("compile_with_namespaces"), "{said}");
    }

    #[test]
    fn other_expression_errors_are_passed_through() {
        // Those messages already say where in the expression the
        // problem is; rewriting them would lose that.
        let err = oxml::XPath::compile("//[").expect_err("malformed");
        let said = super::explain_xpath_error(&err);
        assert!(said.starts_with("bad XPath: "), "{said}");
        assert!(!said.contains("--ns"), "{said}");
    }

    #[test]
    fn a_binding_is_collected() {
        let got = parse_namespace_bindings(&args(&[
            "query", "-n", "m=urn:u", "//m:x",
        ]))
        .expect("valid");
        assert_eq!(got, vec![("m".to_owned(), "urn:u".to_owned())]);
    }

    #[test]
    fn the_long_form_works_too() {
        let got = parse_namespace_bindings(&args(&["--ns", "m=urn:u"]))
            .expect("valid");
        assert_eq!(got.len(), 1);
    }

    #[test]
    fn bindings_accumulate_and_later_ones_win() {
        // Later wins, so a wrapper script can set defaults a caller
        // overrides on the same command line.
        let got = parse_namespace_bindings(&args(&[
            "-n",
            "a=urn:one",
            "-n",
            "b=urn:two",
            "-n",
            "a=urn:three",
        ]))
        .expect("valid");
        assert_eq!(
            got,
            vec![
                ("b".to_owned(), "urn:two".to_owned()),
                ("a".to_owned(), "urn:three".to_owned()),
            ]
        );
    }

    #[test]
    fn a_uri_may_contain_an_equals_sign() {
        // Split on the *first* `=`; a URI with a query string is
        // ordinary and must not be truncated.
        let got = parse_namespace_bindings(&args(&["-n", "m=urn:u?a=b&c=d"]))
            .expect("valid");
        assert_eq!(got[0].1, "urn:u?a=b&c=d");
    }

    #[test]
    fn an_empty_uri_is_allowed() {
        // Binding a prefix to the empty string is unusual but not a
        // usage error; the library decides what it means.
        let got =
            parse_namespace_bindings(&args(&["-n", "m="])).expect("valid");
        assert_eq!(got[0].1, "");
    }

    #[test]
    fn a_binding_without_an_equals_sign_is_rejected() {
        let err = parse_namespace_bindings(&args(&["-n", "bogus"]))
            .expect_err("no `=`");
        assert!(err.contains("PREFIX=URI"), "{err}");
    }

    #[test]
    fn a_binding_needs_a_prefix() {
        let err = parse_namespace_bindings(&args(&["-n", "=urn:u"]))
            .expect_err("no prefix");
        assert!(err.contains("prefix"), "{err}");
    }

    #[test]
    fn the_xml_prefix_may_not_be_rebound() {
        // Bound by the specification. Rebinding it is not something a
        // document can do either.
        let err = parse_namespace_bindings(&args(&["-n", "xml=urn:u"]))
            .expect_err("reserved");
        assert!(err.contains("xml"), "{err}");
    }

    #[test]
    fn a_trailing_flag_with_no_value_is_rejected() {
        let err = parse_namespace_bindings(&args(&["query", "//x", "-n"]))
            .expect_err("no value");
        assert!(err.contains("PREFIX=URI"), "{err}");
    }

    #[test]
    fn the_next_argument_is_taken_even_if_it_looks_like_a_flag() {
        // `-n -t` is a mistake worth reporting rather than silently
        // treating `-t` as a separate option and consuming the
        // argument after it.
        let err = parse_namespace_bindings(&args(&["-n", "-t", "//x"]))
            .expect_err("flag as value");
        assert!(err.contains("PREFIX=URI"), "{err}");
    }

    #[test]
    fn no_bindings_is_not_an_error() {
        let got =
            parse_namespace_bindings(&args(&["query", "//x"])).expect("valid");
        assert!(got.is_empty());
    }
}
