// SPDX-License-Identifier: MIT OR Apache-2.0
//! Resource failures must be reported through the public command API.

struct Input(std::path::PathBuf);

impl Input {
    fn new(label: &str, text: &str) -> Self {
        let path = std::env::temp_dir()
            .join(format!("oxml-cli-budget-{label}-{}", std::process::id()));
        std::fs::write(&path, text).expect("write input fixture");
        Self(path)
    }

    fn argument(&self) -> String {
        self.0.to_string_lossy().into_owned()
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[test]
fn query_refuses_a_string_past_its_evaluation_budget() {
    let text = format!(
        "<r>{}<n/>{}</r>",
        "x".repeat(512 * 1024),
        "x".repeat(512 * 1024 + 1)
    );
    let input = Input::new("query", &text);
    let args = vec!["query".into(), "string(/r)".into(), input.argument()];
    let result = oxml_cli::run(&args, &mut Vec::new(), &mut Vec::new());
    assert!(result.is_err(), "untrusted query ignored the string budget");
}

#[test]
fn input_reading_refuses_a_document_past_its_source_budget() {
    let text = format!("<r>{}</r>", "x".repeat(8 * 1024 * 1024));
    let input = Input::new("input", &text);
    let args = vec!["check".into(), input.argument()];
    let result = oxml_cli::run(&args, &mut Vec::new(), &mut Vec::new());
    assert!(result.is_err(), "input reading ignored the source budget");
}
