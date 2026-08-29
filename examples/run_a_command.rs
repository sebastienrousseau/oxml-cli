//! Drive the CLI the way `main` does, but in-process.
//!
//! `oxml_cli::run` takes its output streams as parameters precisely so
//! a caller can read back what the command produced without spawning a
//! subprocess. This example is that caller.

use std::process::ExitCode;

fn main() {
    let document = std::env::temp_dir().join("oxml-cli-example.xml");
    std::fs::write(&document, "<catalogue><book>Dune</book></catalogue>")
        .expect("writing the example document");

    let args = vec![
        "query".to_owned(),
        "//book".to_owned(),
        document.to_string_lossy().into_owned(),
        "--text".to_owned(),
    ];

    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = oxml_cli::run(&args, &mut out, &mut err)
        .expect("the command should run");

    print!("{}", String::from_utf8_lossy(&out));
    assert_eq!(
        String::from_utf8_lossy(&out).trim(),
        "Dune",
        "querying //book --text should print the element's text"
    );
    assert!(matches!(code, c if c == ExitCode::SUCCESS));

    std::fs::remove_file(&document).expect("removing the example document");
}
