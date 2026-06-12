#![allow(non_snake_case)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    Ok(())
}

#[test]
fn policy__boundaries__api_boundaries__api_v1_surface_has_no_id_catalog(
) -> Result<(), Box<dyn std::error::Error>> {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf();
    let v1_dir = root.join("crates").join("bijux-dna-api").join("src").join("v1");
    let mut files = Vec::new();
    collect_rs_files(&v1_dir, &mut files)?;
    let mut violations = Vec::new();
    for file in files {
        let content = fs::read_to_string(&file)?;
        for token in ["fastq.", "bam.", "qc_post", "retention", "adapter"] {
            if content.contains(token) {
                violations.push(format!("{} contains stage token {token}", file.display()));
            }
        }
    }
    if violations.is_empty() {
        return Ok(());
    }
    Err(format!(
        "API v1 surface must not embed stage IDs or domain tokens:\n{}",
        violations.join("\n")
    )
    .into())
}

#[test]
fn policy__boundaries__api_boundaries__cli_does_not_depend_on_planner_or_engine(
) -> Result<(), Box<dyn std::error::Error>> {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf();
    let cli_src = root.join("crates").join("bijux-dna").join("src");
    let mut files = Vec::new();
    collect_rs_files(&cli_src, &mut files)?;
    let mut violations = Vec::new();
    for file in files {
        let content = fs::read_to_string(&file)?;
        for token in ["bijux_planner", "bijux_dna_engine", "bijux_dna_runner"] {
            if content.contains(token) {
                violations.push(format!("{} contains forbidden token {token}", file.display()));
            }
        }
    }
    if violations.is_empty() {
        return Ok(());
    }
    Err(format!("CLI must only call API surface:\n{}", violations.join("\n")).into())
}
