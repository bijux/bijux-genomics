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
        "merge.rs",
        "mod.rs",
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
fn cli_fastq_exec_has_no_tool_literals() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root not found")?;
    let root = repo_root.join("crates/bijux-cli/src/fastq_exec");
    let mut files = Vec::new();
    collect_rs_files(&root, &mut files);
    let forbidden = [
        "fastp",
        "fastqc",
        "multiqc",
        "seqkit",
        "prinseq",
        "pear",
        "trimmomatic",
        "cutadapt",
        "trim_galore",
        "fastqvalidator",
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
        "fastq_exec must not hardcode tool ids: {offenders:?}"
    );
    Ok(())
}

#[test]
fn cli_fastq_exec_has_no_execution_tooling() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root not found")?;
    let root = repo_root.join("crates/bijux-cli/src/fastq_exec");
    let mut files = Vec::new();
    collect_rs_files(&root, &mut files);
    let forbidden = [
        "std::process::Command",
        "process::Command",
        "Command::new",
        "DockerRunner",
        "docker::",
        "docker_runner",
        "run_docker",
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
        "fastq_exec must not import execution tooling: {offenders:?}"
    );
    Ok(())
}

#[test]
fn cli_fastq_exec_size_guardrail() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root not found")?;
    let root = repo_root.join("crates/bijux-cli/src/fastq_exec");
    let mut files = Vec::new();
    collect_rs_files(&root, &mut files);
    let max_lines = 350usize;
    let mut offenders = Vec::new();
    for path in files {
        let contents = fs::read_to_string(&path)?;
        let line_count = contents.lines().count();
        if line_count > max_lines {
            offenders.push(format!(
                "{} -> {} lines (max {})",
                path.display(),
                line_count,
                max_lines
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "fastq_exec files must stay small: {offenders:?}"
    );
    Ok(())
}

#[test]
fn cli_fastq_exec_imports_are_thin() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root not found")?;
    let root = repo_root.join("crates/bijux-cli/src/fastq_exec");
    let mut files = Vec::new();
    collect_rs_files(&root, &mut files);
    let allowed_prefixes = [
        "std::",
        "crate::",
        "bijux_engine::api",
        "bijux_stages_fastq",
        "bijux_core",
        "clap",
        "correct::",
        "filter::",
        "merge::",
        "preprocess::",
        "preprocess_exec::",
        "qc_post::",
        "screen::",
        "stats_neutral::",
        "trim::",
        "umi::",
        "validate_pre::",
    ];
    let mut offenders = Vec::new();
    for path in files {
        let contents = fs::read_to_string(&path)?;
        for line in contents.lines() {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix("use ") {
                let path_part = rest.split([';', ' ']).next().unwrap_or("");
                if !allowed_prefixes
                    .iter()
                    .any(|prefix| path_part.starts_with(prefix))
                {
                    offenders.push(format!("{} -> {}", path.display(), path_part));
                }
            }
            if let Some(rest) = trimmed.strip_prefix("pub use ") {
                let path_part = rest.split([';', ' ']).next().unwrap_or("");
                if !allowed_prefixes
                    .iter()
                    .any(|prefix| path_part.starts_with(prefix))
                {
                    offenders.push(format!("{} -> {}", path.display(), path_part));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "fastq_exec imports must be thin CLI adapters: {offenders:?}"
    );
    Ok(())
}

#[test]
fn cli_fastq_exec_size_or_no_exec_imports() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root not found")?;
    let root = repo_root.join("crates/bijux-cli/src/fastq_exec");
    let mut files = Vec::new();
    collect_rs_files(&root, &mut files);
    let max_lines = 450usize;
    let mut offenders = Vec::new();
    for path in files {
        let contents = fs::read_to_string(&path)?;
        let line_count = contents.lines().count();
        if line_count > max_lines {
            let has_env = contents.contains("bijux_env_runtime::")
                || contents.contains("bijux_env_builder::");
            let has_executor = contents.contains("services::executor");
            if has_env || has_executor {
                offenders.push(format!(
                    "{} -> {} lines (env={}, executor={})",
                    path.display(),
                    line_count,
                    has_env,
                    has_executor
                ));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "large fastq_exec modules must not import executor/environment: {offenders:?}"
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

#[test]
fn cli_does_not_import_engine_internals() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root not found")?;
    let root = repo_root.join("crates/bijux-cli/src");
    let mut files = Vec::new();
    collect_rs_files(&root, &mut files);
    let forbidden = ["bijux_engine::services", "bijux_engine::core"];
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
        "CLI must not import engine internals: {offenders:?}"
    );
    Ok(())
}
