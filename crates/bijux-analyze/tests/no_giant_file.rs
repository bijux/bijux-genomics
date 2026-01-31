use std::fs;
use std::path::PathBuf;

fn collect_rs_files(dir: &PathBuf, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

#[test]
fn analyze_sources_not_giant() -> std::io::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files)?;
    let max_lines = 2000usize;
    for path in files {
        let content = fs::read_to_string(&path)?;
        let lines = content.lines().count();
        assert!(
            lines <= max_lines,
            "{} has {} lines (max {})",
            path.display(),
            lines,
            max_lines
        );
    }
    Ok(())
}
