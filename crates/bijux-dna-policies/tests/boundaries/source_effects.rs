#![allow(non_snake_case)]

use std::path::{Path, PathBuf};

#[path = "../support/rust_source/tokens.rs"]
mod rust_source_tokens;

#[test]
fn policy__boundaries__source_effects__production_source_does_not_spawn_processes_or_open_networks()
{
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let findings = forbidden_source_tokens(
        &root.join("src"),
        &[
            "Command::new",
            ".spawn(",
            "std::process",
            "tokio::process",
            "std::net::",
            "TcpStream",
            "UdpSocket",
            "reqwest::",
            "ureq::",
            "hyper::",
            "tokio::net",
        ],
    );

    assert!(
        findings.is_empty(),
        "policy production source must not spawn processes or open network connections:\n{}",
        findings.join("\n")
    );
}

#[test]
fn policy__boundaries__source_effects__production_source_does_not_mutate_filesystem_outputs() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let findings = forbidden_source_tokens(
        &root.join("src"),
        &["std::fs::write", "fs::write", "File::create", "create_dir", "remove_file", "remove_dir"],
    );

    assert!(
        findings.is_empty(),
        "policy production source must inspect files without mutating the filesystem:\n{}",
        findings.join("\n")
    );
}

fn forbidden_source_tokens(root: &Path, forbidden_tokens: &[&str]) -> Vec<String> {
    source_files(root)
        .into_iter()
        .flat_map(|path| findings_in_file(&path, forbidden_tokens))
        .collect()
}

fn source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_source_files(root, &mut files);
    files.sort();
    files
}

fn collect_source_files(path: &Path, files: &mut Vec<PathBuf>) {
    for entry in
        std::fs::read_dir(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
    {
        let entry = entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_source_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn findings_in_file(path: &Path, forbidden_tokens: &[&str]) -> Vec<String> {
    let content = bijux_dna_testkit::workspace_support::strip_rust_test_modules(
        &std::fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("read {}: {err}", path.display())),
    );
    findings_in_content(path, &content, forbidden_tokens)
}

fn executable_source(content: &str) -> Option<String> {
    let tokens = rust_source_tokens::source_tokens(content)?;
    let mut executable =
        content.bytes().map(|byte| if byte == b'\n' { b'\n' } else { b' ' }).collect::<Vec<_>>();
    for token in tokens.into_iter().filter(|token| !token.literal) {
        // The lexer returns subslices of this input; preserve their original byte positions.
        let start = token.text.as_ptr().addr().checked_sub(content.as_ptr().addr())?;
        let end = start.checked_add(token.text.len())?;
        executable.get_mut(start..end)?.copy_from_slice(token.text.as_bytes());
    }
    String::from_utf8(executable).ok()
}

fn findings_in_content(path: &Path, content: &str, forbidden_tokens: &[&str]) -> Vec<String> {
    let Some(content) = executable_source(content) else {
        return vec![format!("{} cannot classify Rust source", path.display())];
    };
    content
        .lines()
        .enumerate()
        .flat_map(|(line_index, line)| {
            forbidden_tokens.iter().filter(move |token| line.contains(**token)).map(move |token| {
                format!("{}:{} contains `{token}`", path.display(), line_index + 1)
            })
        })
        .collect()
}

#[test]
fn policy__boundaries__source_effects__inline_tests_do_not_count_as_production_effects() {
    let source = r#"
pub fn policy() {}

#[cfg(test)]
mod tests {
    fn exercises_fixture() {
        std::fs::write("fixture", "value").unwrap();
    }
}
"#;
    let production = bijux_dna_testkit::workspace_support::strip_rust_test_modules(source);
    assert!(
        findings_in_content(Path::new("policy.rs"), &production, &["std::fs::write"]).is_empty()
    );
    assert_eq!(findings_in_content(Path::new("policy.rs"), source, &["std::fs::write"]).len(), 1);

    let declarations = r##"
const RULES: &[&str] = &["Command::new", "std::net::TcpStream", "std::fs::write"];
const RAW: &str = r#"Command::new("tool").spawn(); std::fs::write("path", []);"#;
const BYTES: &[u8] = b"std::net::TcpStream";
// Command::new("tool").spawn(); std::fs::write("path", []);
/* std::net::TcpStream; /* std::fs::write("path", []); */ */
"##;
    let effects = ["Command::new", ".spawn(", "std::net::", "std::fs::write"];
    assert!(findings_in_content(Path::new("rules.rs"), declarations, &effects).is_empty());
    for (source, expected) in [
        (r#"fn execute() { Command::new("tool").spawn(); }"#, "Command::new"),
        (r#"fn execute() { Command::new("tool").spawn(); }"#, ".spawn("),
        (r#"fn connect() { std::net::TcpStream::connect("host:1"); }"#, "std::net::"),
        (r#"fn publish() { std::fs::write("path", []); }"#, "std::fs::write"),
    ] {
        assert_eq!(findings_in_content(Path::new("production.rs"), source, &[expected]).len(), 1);
    }
    let mixed = r#"const RULE: &str = "std::fs::write";
fn publish() { std::fs::write("path", []); }"#;
    let mixed_findings = findings_in_content(Path::new("mixed.rs"), mixed, &["std::fs::write"]);
    assert_eq!(mixed_findings, ["mixed.rs:2 contains `std::fs::write`"]);
    assert_eq!(
        findings_in_content(
            Path::new("invalid.rs"),
            r#"const RULE: &str = "unterminated"#,
            &effects
        ),
        ["invalid.rs cannot classify Rust source"]
    );
}
