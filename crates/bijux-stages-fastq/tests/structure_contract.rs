use std::path::Path;

#[test]
fn stages_fastq_rs_file_counts_are_bounded() -> anyhow::Result<()> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&src).min_depth(0).max_depth(6) {
        let entry = entry?;
        if !entry.file_type().is_dir() {
            continue;
        }
        let mut count = 0usize;
        for child in std::fs::read_dir(entry.path())? {
            let child = child?;
            let path = child.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name == "lib.rs" || name == "main.rs" || name == "mod.rs" {
                continue;
            }
            count += 1;
        }
        if count > 10 {
            offenders.push((entry.path().display().to_string(), count));
        }
    }
    assert!(
        offenders.is_empty(),
        "rs file count exceeded in dirs: {offenders:?}"
    );
    Ok(())
}
