// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml. All rights reserved.

//! End-to-end behaviour of the binary.
//!
//! These drive the compiled binary rather than calling functions,
//! because what matters here is the contract a shell script sees:
//! stdout, stderr, and the exit status.

use std::io::Write as _;
use std::process::{Command, Stdio};

const DOC: &str = r#"<library>
  <book lang="en"><title>Dune</title></book>
  <book lang="fr"><title>Germinal</title></book>
</library>"#;

struct Output {
    stdout: String,
    stderr: String,
    code: i32,
}

fn run(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_oxml"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("binary runs");
    // A broken pipe here is not a failure. `oxml frobnicate` reports an
    // unknown command and exits *without reading stdin*, so the write
    // races the child's exit -- Linux reports EPIPE, macOS usually
    // absorbs it into the buffer, and the test passed on one platform
    // and not the other.
    let _ = child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(stdin.as_bytes());
    let out = child.wait_with_output().expect("wait");
    Output {
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        code: out.status.code().unwrap_or(-1),
    }
}

#[test]
fn query_prints_matched_text() {
    let out = run(&["query", "//title", "-t"], DOC);
    assert_eq!(out.code, 0);
    assert_eq!(out.stdout, "Dune\nGerminal\n");
}

#[test]
fn query_counts_with_the_flag() {
    let out = run(&["query", "//book", "-c"], DOC);
    assert_eq!(out.code, 0);
    assert_eq!(out.stdout.trim(), "2");
}

/// A non-node-set expression should behave like the expression it is,
/// not be forced into a node listing.
#[test]
fn a_scalar_expression_prints_its_value() {
    let out = run(&["query", "count(//book)"], DOC);
    assert_eq!(out.code, 0);
    assert_eq!(out.stdout.trim(), "2");
}

/// Exit 1 on no matches is what makes `oxml query ... && ...` useful
/// in a shell.
#[test]
fn no_matches_exits_nonzero() {
    let out = run(&["query", "//nothing", "-c"], DOC);
    assert_eq!(out.code, 1);
    assert_eq!(out.stdout.trim(), "0");
}

#[test]
fn check_reports_well_formedness() {
    assert_eq!(run(&["check"], DOC).code, 0);

    let bad = run(&["check"], "<a><b></a>");
    assert_eq!(bad.code, 1);
    assert!(bad.stderr.contains("not well-formed"), "{}", bad.stderr);
    // The position must be usable, not just "somewhere".
    assert!(bad.stderr.contains(':'), "{}", bad.stderr);
}

#[test]
fn stats_summarises_the_document() {
    let out = run(&["stats"], DOC);
    assert_eq!(out.code, 0);
    assert!(out.stdout.contains("elements"));
    assert!(out.stdout.contains("max depth"));
}

#[test]
fn a_bad_xpath_is_a_usage_error_not_a_crash() {
    let out = run(&["query", "//book["], DOC);
    assert_eq!(out.code, 2);
    assert!(out.stderr.contains("bad XPath"), "{}", out.stderr);
}

#[test]
fn an_unknown_command_explains_itself() {
    let out = run(&["frobnicate"], DOC);
    assert_eq!(out.code, 2);
    assert!(out.stderr.contains("unknown command"), "{}", out.stderr);
}

#[test]
fn help_and_version_succeed() {
    assert_eq!(run(&["--help"], "").code, 0);
    let v = run(&["--version"], "");
    assert_eq!(v.code, 0);
    assert!(v.stdout.starts_with("oxml "), "{}", v.stdout);
}

/// Diagnostics belong on stderr so the tool composes in a pipeline.
#[test]
fn validation_violations_go_to_stderr() {
    let dir = std::env::temp_dir().join("oxml_cli_test");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let xsd = dir.join("s.xsd");
    std::fs::write(
        &xsd,
        r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
             <xs:element name="library">
               <xs:complexType><xs:sequence>
                 <xs:element name="book" maxOccurs="unbounded">
                   <xs:complexType><xs:sequence>
                     <xs:element name="title" type="xs:string"/>
                   </xs:sequence>
                   <xs:attribute name="lang" use="required"/>
                   </xs:complexType>
                 </xs:element>
               </xs:sequence></xs:complexType>
             </xs:element>
           </xs:schema>"#,
    )
    .expect("write schema");

    let ok = run(&["validate", xsd.to_str().unwrap()], DOC);
    assert_eq!(ok.code, 0, "stderr: {}", ok.stderr);
    assert_eq!(ok.stdout.trim(), "valid");

    let bad = run(
        &["validate", xsd.to_str().unwrap()],
        "<library><book><title>x</title></book></library>",
    );
    assert_eq!(bad.code, 1);
    assert!(bad.stdout.is_empty(), "stdout was {:?}", bad.stdout);
    assert!(bad.stderr.contains("lang"), "{}", bad.stderr);
}

/// Write `contents` to a uniquely named file and return its path.
fn temp_file(name: &str, contents: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("oxml-cli-test-{name}.xml"));
    std::fs::write(&path, contents).expect("write temp file");
    path
}

const RICH: &str = r#"<library count="2">
  <!-- a comment that is here to be described -->
  <book lang="en" year="1965"><title>Dune</title></book>
  <empty/>
  <long>aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa</long>
</library>"#;

#[test]
fn the_default_query_output_describes_nodes() {
    // Without `--text` the output is a summary, not the text content.
    // This is the shape a person reads, and nothing covered it.
    let out = run(&["query", "//book"], RICH);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(out.stdout.contains("<book"), "{}", out.stdout);
    assert!(out.stdout.contains("lang=\"en\""), "{}", out.stdout);
    assert!(out.stdout.contains("Dune"), "{}", out.stdout);
}

#[test]
fn an_empty_element_is_described_as_self_closing() {
    let out = run(&["query", "//empty"], RICH);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(out.stdout.contains("<empty/>"), "{}", out.stdout);
}

#[test]
fn attributes_are_described_as_name_and_value() {
    let out = run(&["query", "//book/@lang"], RICH);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(out.stdout.contains("lang=\"en\""), "{}", out.stdout);
}

#[test]
fn comments_are_described() {
    let out = run(&["query", "//comment()"], RICH);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(out.stdout.contains("<!--"), "{}", out.stdout);
    assert!(out.stdout.contains("comment"), "{}", out.stdout);
}

#[test]
fn text_nodes_are_described() {
    let out = run(&["query", "//title/text()"], RICH);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(out.stdout.contains("Dune"), "{}", out.stdout);
}

#[test]
fn long_content_is_truncated_with_an_ellipsis() {
    // Otherwise one long element floods the terminal.
    let out = run(&["query", "//long"], RICH);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(out.stdout.contains('…'), "{}", out.stdout);
    let line = out.stdout.lines().next().expect("a line");
    assert!(line.chars().count() < 100, "not truncated: {line}");
}

#[test]
fn short_content_is_not_truncated() {
    let out = run(&["query", "--text", "//title"], RICH);
    assert_eq!(out.stdout.trim(), "Dune");
    assert!(!out.stdout.contains('…'), "{}", out.stdout);
}

#[test]
fn a_file_argument_is_read_instead_of_stdin() {
    let path = temp_file("read", RICH);
    let out = run(
        &["query", "--text", "//title", path.to_str().expect("path")],
        "",
    );
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert_eq!(out.stdout.trim(), "Dune");
    drop(std::fs::remove_file(&path));
}

#[test]
fn an_unreadable_file_is_a_usage_error() {
    // Exit 2, distinct from 1 which means "the document failed the
    // check" — a script has to be able to tell those apart.
    let out = run(&["check", "/nonexistent/path/nothing.xml"], "");
    assert_eq!(out.code, 2, "{} {}", out.stdout, out.stderr);
    assert!(out.stderr.contains("cannot read"), "{}", out.stderr);
}

#[test]
fn a_parse_error_reports_line_and_column() {
    // The position is the whole value of the message.
    let out = run(&["query", "//a"], "<a><b></a>");
    assert_eq!(out.code, 2, "{} {}", out.stdout, out.stderr);
    let msg = out.stderr.trim();
    // Expect a `line:column:` pair somewhere in the message, whatever
    // prefix the binary puts in front of it.
    let has_position = msg.split_whitespace().any(|tok| {
        let mut it = tok.trim_end_matches(':').split(':');
        matches!(
            (
                it.next().map(str::parse::<u32>),
                it.next().map(str::parse::<u32>)
            ),
            (Some(Ok(_)), Some(Ok(_)))
        )
    });
    assert!(has_position, "no line:column in: {msg}");
}

#[test]
fn counting_an_empty_result_exits_nonzero() {
    // `-c` prints 0 and still signals "nothing matched" via the status,
    // so `if oxml query -c ...` works in a shell.
    let out = run(&["query", "-c", "//missing"], RICH);
    assert_eq!(out.stdout.trim(), "0");
    assert_eq!(out.code, 1);
}

#[test]
fn stats_counts_comments() {
    let out = run(&["stats"], RICH);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(
        out.stdout.to_lowercase().contains("comment"),
        "{}",
        out.stdout
    );
}

#[test]
fn an_empty_node_set_prints_nothing_and_exits_one() {
    // Without `-c` there is no output at all, so the exit status is the
    // only signal a caller has.
    let out = run(&["query", "//missing"], RICH);
    assert_eq!(out.code, 1);
    assert!(out.stdout.is_empty(), "{}", out.stdout);
}

#[test]
fn the_document_node_describes_as_empty_rather_than_panicking() {
    // `/` selects the document root, which is not an element, an
    // attribute, a comment or text — the branch nothing else reaches.
    let out = run(&["query", "/"], RICH);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert_eq!(out.stdout.trim(), "");
}

#[test]
fn every_missing_argument_explains_itself() {
    // Each of these is a distinct message; a generic "usage" would
    // leave the user guessing which argument was wrong.
    for (args, expect) in [
        // A flag with no subcommand: bare `oxml` is documented to print
        // usage and succeed, so this is the path that reaches the error.
        (vec!["-t"], "no command"),
        (vec!["query"], "XPath"),
        (vec!["validate"], ".xsd"),
    ] {
        let out = run(&args, "<a/>");
        assert_eq!(out.code, 2, "{args:?} -> {}", out.stderr);
        assert!(
            out.stderr.to_lowercase().contains(&expect.to_lowercase()),
            "{args:?} said {:?}, expected mention of {expect}",
            out.stderr
        );
    }
}

#[test]
fn each_subcommand_accepts_a_file_path() {
    // Every subcommand takes its document from a path as well as from
    // stdin, and the path lands in a different argument slot for each.
    let doc = temp_file("subcommands", RICH);
    let p = doc.to_str().expect("path");

    let stats = run(&["stats", p], "");
    assert_eq!(stats.code, 0, "{}", stats.stderr);
    assert!(stats.stdout.contains("elements"), "{}", stats.stdout);

    let check = run(&["check", p], "");
    assert_eq!(check.code, 0, "{}", check.stderr);

    let xsd = temp_file(
        "schema",
        r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
             <xs:element name="note" type="xs:string"/>
           </xs:schema>"#,
    );
    let note = temp_file("note", "<note>hi</note>");
    let validate = run(
        &[
            "validate",
            xsd.to_str().expect("path"),
            note.to_str().expect("path"),
        ],
        "",
    );
    assert_eq!(validate.code, 0, "{} {}", validate.stdout, validate.stderr);

    for f in [&doc, &xsd, &note] {
        drop(std::fs::remove_file(f));
    }
}

#[test]
fn a_missing_schema_file_is_a_usage_error() {
    let out = run(&["validate", "/nonexistent/schema.xsd"], "<a/>");
    assert_eq!(out.code, 2, "{} {}", out.stdout, out.stderr);
    assert!(!out.stderr.is_empty());
}

#[test]
fn a_bare_invocation_prints_usage_and_succeeds() {
    // Running the tool with nothing is a request for help, not an
    // error — it must not exit non-zero.
    let out = run(&[], "");
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(out.stdout.contains("USAGE"), "{}", out.stdout);
    assert!(out.stderr.is_empty(), "{}", out.stderr);
}

/// `--ns` binds a prefix for the query expression.
///
/// Untested until now, and the README paid for it: the "Not yet" list
/// claimed namespace prefixes on the command line were unimplemented
/// while the options table two hundred lines above documented `--ns`
/// and the unbound-prefix error told you to use it. A feature nothing
/// exercises is a feature the documentation stops believing in.
#[test]
fn ns_binds_a_prefix_for_the_query() {
    const NS: &str = r#"<r xmlns:m="urn:x"><m:a>hit</m:a><b>miss</b></r>"#;

    let out = run(&["query", "--ns", "m=urn:x", "//m:a", "-t"], NS);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert_eq!(out.stdout, "hit\n");

    // The short form is the same flag.
    let short = run(&["query", "-n", "m=urn:x", "//m:a", "-t"], NS);
    assert_eq!(short.stdout, "hit\n", "{}", short.stderr);
}

/// An unbound prefix is refused, and the message says how to fix it.
#[test]
fn an_unbound_prefix_is_refused_with_the_remedy() {
    const NS: &str = r#"<r xmlns:m="urn:x"><m:a>hit</m:a></r>"#;
    let out = run(&["query", "//m:a", "-t"], NS);
    assert_ne!(out.code, 0, "an unbound prefix must not silently match");
    assert!(
        out.stderr.contains("--ns"),
        "the error should name the flag that fixes it: {}",
        out.stderr
    );
}

/// The flag is repeatable, which is the only way to query across two
/// namespaces at once.
#[test]
fn ns_is_repeatable() {
    const TWO: &str = r#"<r xmlns:m="urn:x" xmlns:n="urn:y">
        <m:a>one</m:a><n:b>two</n:b></r>"#;
    let out = run(
        &[
            "query",
            "--ns",
            "m=urn:x",
            "--ns",
            "n=urn:y",
            "//m:a|//n:b",
            "-t",
        ],
        TWO,
    );
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert_eq!(out.stdout, "one\ntwo\n");
}

/// A prefix bound to the wrong URI matches nothing.
///
/// The binding is by URI, not by spelling: matching on the prefix
/// would make `--ns` decorative.
#[test]
fn a_prefix_bound_to_the_wrong_uri_matches_nothing() {
    const NS: &str = r#"<r xmlns:m="urn:x"><m:a>hit</m:a></r>"#;
    let out = run(&["query", "--ns", "m=urn:WRONG", "//m:a", "-c"], NS);
    assert_eq!(out.stdout.trim(), "0", "{}", out.stderr);
}
