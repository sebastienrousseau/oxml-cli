// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml. All rights reserved.

//! What each subcommand costs, without a process in the way.
//!
//! A CLI's felt latency is process spawn plus the work. Spawn is the
//! operating system's business and swamps everything else on a small
//! document, so what is measured here is the second part: `run`
//! driving a real command with its output going to a buffer instead of
//! a terminal.
//!
//! Reported per invocation, because that is the unit a user waits for.
//!
//! Absolute figures describe the machine as much as the code -- see
//! `oxml`'s `doc/BENCHMARKS.md`. Compare runs, not numbers.

use std::fmt::Write as _;
use std::hint::black_box;
use std::io::Write;
use std::time::Instant;

/// A document of `n` entries, written to a temporary file.
///
/// The commands take a path, so the benchmark has to supply one. The
/// read is part of what a user waits for, so it stays in.
struct Document(std::path::PathBuf);

impl Document {
    fn new(name: &str, entries: usize) -> Self {
        let mut s = String::from("<?xml version=\"1.0\"?>\n<catalogue>\n");
        for i in 0..entries {
            let _ = write!(
                s,
                "  <book id=\"b{i}\" lang=\"en\">\n    \
                 <title>Title {i}</title>\n    \
                 <pages>{i}</pages>\n  </book>\n"
            );
        }
        s.push_str("</catalogue>\n");
        let path = std::env::temp_dir().join(name);
        let mut f = std::fs::File::create(&path).expect("a temporary file");
        f.write_all(s.as_bytes()).expect("write");
        Self(path)
    }

    fn path(&self) -> String {
        self.0.display().to_string()
    }
}

impl Drop for Document {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// The fastest of `rounds` runs.
///
/// Contention can only make a run slower, so the fastest is the least
/// perturbed sample. A mean would mostly measure whatever else the
/// machine was doing.
fn fastest(rounds: usize, mut f: impl FnMut()) -> f64 {
    let mut best = f64::INFINITY;
    for _ in 0..rounds {
        let start = Instant::now();
        f();
        best = best.min(start.elapsed().as_secs_f64());
    }
    best
}

fn main() {
    let small = Document::new("oxml-bench-small.xml", 10);
    let large = Document::new("oxml-bench-large.xml", 2_000);

    let cases: Vec<(String, Vec<String>)> = vec![
        // No document at all: argument handling on its own, which is
        // the floor every other row sits on.
        ("--help".to_owned(), vec!["--help".to_owned()]),
        (
            "check (10 entries)".to_owned(),
            vec!["check".to_owned(), small.path()],
        ),
        (
            "check (2,000 entries)".to_owned(),
            vec!["check".to_owned(), large.path()],
        ),
        (
            "stats (2,000 entries)".to_owned(),
            vec!["stats".to_owned(), large.path()],
        ),
        (
            "query //title (2,000)".to_owned(),
            vec!["query".to_owned(), "//title".to_owned(), large.path()],
        ),
        (
            "query -t //title (2,000)".to_owned(),
            vec![
                "query".to_owned(),
                "-t".to_owned(),
                "//title".to_owned(),
                large.path(),
            ],
        ),
        (
            "query count() (2,000)".to_owned(),
            vec![
                "query".to_owned(),
                "count(//title)".to_owned(),
                large.path(),
            ],
        ),
    ];

    println!("per invocation, fastest of 20 rounds, spawn excluded\n");
    println!("{:<28} {:>12}", "command", "time");
    for (name, argv) in &cases {
        let reps = if argv.len() < 2 { 200 } else { 1 };
        let seconds = fastest(20, || {
            for _ in 0..reps {
                let mut out = Vec::with_capacity(1 << 16);
                let mut err = Vec::new();
                let _ = black_box(oxml_cli::run(
                    black_box(argv),
                    &mut out,
                    &mut err,
                ));
                let _ = black_box(out);
            }
        }) / f64::from(u16::try_from(reps).unwrap_or(1));
        println!("{name:<28} {:>9.1} us", seconds * 1e6);
    }

    // How much of `check` is the parse, paired so that both meet the
    // same machine conditions. Timing them in separate loops and
    // dividing gave impossible answers in the sibling crate: on a busy
    // machine the runs disagree by more than the difference.
    let source = std::fs::read_to_string(large.path()).expect("readable");
    let argv = vec!["check".to_owned(), large.path()];
    let (mut whole, mut parse) = (f64::INFINITY, f64::INFINITY);
    for _ in 0..40 {
        let a = Instant::now();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let _ = black_box(oxml_cli::run(&argv, &mut out, &mut err));
        whole = whole.min(a.elapsed().as_secs_f64());
        let b = Instant::now();
        let _ = black_box(oxml::parse(black_box(&source)));
        parse = parse.min(b.elapsed().as_secs_f64());
    }
    println!(
        "\n`check` against the parse inside it, paired:\n  \
         {:.0} us vs {:.0} us -- {:+.1}% for argument handling and file \
         I/O",
        whole * 1e6,
        parse * 1e6,
        (whole / parse - 1.0) * 100.0
    );
}
