use std::path::PathBuf;

fn repo_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    match manifest_dir.parent().and_then(|p| p.parent()) {
        Some(root) => Ok(root.to_path_buf()),
        None => Err("repo root not found".into()),
    }
}

#[test]
fn stages_may_import_domain_fastq_but_not_engine() -> Result<(), Box<dyn std::error::Error>> {
    let root = repo_root()?;
    let src_dir = root.join("crates/bijux-stages/src");
    let mut bad = Vec::new();
    for entry in walkdir::WalkDir::new(&src_dir) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let contents = std::fs::read_to_string(entry.path())?;
        if contents.contains("bijux_engine") {
            bad.push(entry.path().display().to_string());
        }
    }
    assert!(
        bad.is_empty(),
        "bijux-stages must not import bijux_engine: {bad:?}"
    );
    Ok(())
}
