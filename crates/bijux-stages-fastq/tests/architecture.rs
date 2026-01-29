use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn collect_rs_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.path().to_path_buf())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .collect()
}

#[test]
fn stages_fastq_has_no_execution_calls() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.join("src");
    let files = collect_rs_files(&root);
    let forbidden = [
        "std::process::Command",
        "process::Command",
        "Command::new",
        "DockerRunner",
        "docker::",
        "docker_runner",
        "RunnerKind",
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
        "stages-fastq must not execute tools directly: {offenders:?}"
    );
    Ok(())
}
