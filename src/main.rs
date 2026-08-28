// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml. All rights reserved.

//! The `oxml` executable.
//!
//! Every command lives in the library, which is what the tests and
//! benchmarks drive. This binary supplies the process's own arguments
//! and streams, and turns a usage error into an exit code.

#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let mut out = stdout.lock();
    let mut err = stderr.lock();
    match oxml_cli::run(&args, &mut out, &mut err) {
        Ok(code) => code,
        Err(e) => {
            let _ = std::io::Write::write_fmt(
                &mut err,
                format_args!("oxml: {e}\n"),
            );
            ExitCode::from(2)
        }
    }
}
