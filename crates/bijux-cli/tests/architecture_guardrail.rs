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
