use std::path::{Path, PathBuf};

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
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
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
