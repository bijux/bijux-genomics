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
fn core_sources_not_giant() -> std::io::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files)?;
    let max_lines = 800usize;
    let max_lines_exceptions = [(src_dir.join("lib.rs"), 1200usize)];
    for path in files {
        let content = fs::read_to_string(&path)?;
        let lines = content.lines().count();
        let mut allowed = max_lines;
        for (exception, limit) in &max_lines_exceptions {
            if &path == exception {
                allowed = *limit;
                break;
            }
        }
        assert!(
            lines <= allowed,
            "{} has {} lines (max {})",
            path.display(),
            lines,
            allowed
        );
    }
    Ok(())
}
