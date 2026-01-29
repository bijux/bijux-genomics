use std::fs;
use std::path::{Path, PathBuf};

fn collect_rs_files(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

#[test]
fn cli_fastq_exec_does_not_introduce_new_modules() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root not found")?;
    let root = repo_root.join("crates/bijux-cli/src/fastq_exec");
    let mut files = Vec::new();
    collect_rs_files(&root, &mut files);
    let mut file_names: Vec<String> = files
        .into_iter()
        .filter_map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
        })
        .collect();
    file_names.sort();

    let expected = vec![
        "correct.rs",
        "filter.rs",
        "helpers.rs",
        "merge.rs",
        "mod.rs",
        "preprocess.rs",
        "preprocess_exec.rs",
        "qc_post.rs",
        "screen.rs",
        "stats_neutral.rs",
        "trim.rs",
        "umi.rs",
        "validate_pre.rs",
    ];
    assert_eq!(
        file_names, expected,
        "fastq_exec module list changed; keep CLI wiring-only"
    );
    Ok(())
}

#[test]
fn cli_does_not_import_domain_fastq() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root not found")?;
    let root = repo_root.join("crates/bijux-cli/src");
    let mut files = Vec::new();
    collect_rs_files(&root, &mut files);
    let mut offenders = Vec::new();
    for path in files {
        let contents = fs::read_to_string(&path)?;
        if contents.contains("bijux_domain_fastq") {
            offenders.push(path.display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "CLI must not import bijux_domain_fastq directly: {offenders:?}"
    );
    Ok(())
}

#[test]
fn cli_fastq_exec_has_no_new_public_fns() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root not found")?;
    let root = repo_root.join("crates/bijux-cli/src/fastq_exec");
    let mut files = Vec::new();
    collect_rs_files(&root, &mut files);
    let expected = [
        "bench_fastq_correct",
        "bench_fastq_filter",
        "bench_fastq_merge",
        "bench_fastq_preprocess",
        "bench_fastq_qc_post",
        "bench_fastq_screen",
        "bench_fastq_stats_neutral",
        "bench_fastq_trim",
        "bench_fastq_umi",
        "bench_fastq_validate_pre",
        "fastq_preprocess_plan",
        "fastq_preprocess_run",
    ];
    let mut offenders = Vec::new();
    for path in files {
        let contents = fs::read_to_string(&path)?;
        for line in contents.lines() {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix("pub fn ") {
                let name = rest
                    .split(|ch: char| ch == '(' || ch.is_whitespace() || ch == '<')
                    .next()
                    .unwrap_or("");
                if !expected.contains(&name) {
                    offenders.push(format!("{}::{name}", path.display()));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "fastq_exec should not add new public functions: {offenders:?}"
    );
    Ok(())
}

#[test]
fn cli_fastq_exec_does_not_match_on_tool_ids() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root not found")?;
    let root = repo_root.join("crates/bijux-cli/src/fastq_exec");
    let mut files = Vec::new();
    collect_rs_files(&root, &mut files);
    let forbidden = [
        "match tool",
        "match tool_id",
        "match tool.as_str()",
        "match tool_id.as_str()",
    ];
    let mut offenders = Vec::new();
    for path in files {
        let contents = fs::read_to_string(&path)?;
        for needle in &forbidden {
            if contents.contains(needle) {
                offenders.push(format!("{} -> {}", path.display(), needle));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "fastq_exec must not match on tool ids: {offenders:?}"
    );
    Ok(())
}

#[test]
fn cli_fastq_exec_avoids_shell_execution() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root not found")?;
    let root = repo_root.join("crates/bijux-cli/src/fastq_exec");
    let mut files = Vec::new();
    collect_rs_files(&root, &mut files);
    let mut offenders = Vec::new();
    for path in files {
        let contents = fs::read_to_string(&path)?;
        if contents.contains("std::process::Command") {
            offenders.push(path.display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "fastq_exec should not invoke shell commands: {offenders:?}"
    );
    Ok(())
}
