#![no_main]
//! Arbitrary arguments must never panic the command line.
//!
//! Argument parsing here is hand-rolled rather than taken from a
//! crate, which is a deliberate trade -- fewer dependencies to audit,
//! but no one else\'s fuzzing to inherit. This target supplies that.
//!
//! Output goes to a buffer rather than the process\'s stdout, which is
//! only possible because the commands take their streams as
//! parameters.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = core::str::from_utf8(data) else {
        return;
    };

    // Treat each line as one argument. This reaches the flag parser,
    // the `--ns PREFIX=URI` splitter and the subcommand dispatch.
    let args: Vec<String> = text.lines().take(16).map(str::to_owned).collect();
    if args.is_empty() {
        return;
    }

    let (mut out, mut err) = (Vec::new(), Vec::new());
    // Errors are expected -- a usage mistake is the normal outcome for
    // random input. A panic is not.
    let _ = oxml_cli::run(&args, &mut out, &mut err);
});
