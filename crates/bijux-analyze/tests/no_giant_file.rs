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

fn collect_all_files(dir: &PathBuf, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_all_files(&path, files)?;
        } else {
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
    let max_lines = 300usize;
    let max_lines_exceptions = [
        (src_dir.join("report").join("build.rs"), 400usize),
        (
            src_dir.join("aggregate").join("registry_defs.rs"),
            1200usize,
        ),
        (src_dir.join("aggregate").join("metrics_fastq.rs"), 600usize),
        (src_dir.join("load").join("sqlite_queries.rs"), 1800usize),
        (src_dir.join("report").join("bench.rs"), 900usize),
    ];
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

#[test]
fn no_tmp_files_in_src() -> std::io::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    let mut files = Vec::new();
    collect_all_files(&src_dir, &mut files)?;
    for path in files {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            assert_ne!(ext, "tmp", "temporary file found: {}", path.display());
        }
    }
    Ok(())
}

#[test]
fn no_deep_modules_in_src() -> std::io::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files)?;
    for path in files {
        let rel = path.strip_prefix(&src_dir).unwrap_or(&path);
        let components: Vec<_> = rel.components().collect();
        if components.len() <= 2 {
            continue;
        }
        let allowed =
            components.len() == 2 && rel.file_name().and_then(|s| s.to_str()) == Some("mod.rs");
        assert!(allowed, "module depth exceeds 2 levels: {}", path.display());
    }
    Ok(())
}

#[test]
fn no_garbage_module_names() -> std::io::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files)?;
    let banned = [
        "part", "tmp", "misc", "stuff", "old", "new", "copy", "final",
    ];
    for path in files {
        let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let stem = file_name.trim_end_matches(".rs");
        for token in &banned {
            assert!(
                !stem.starts_with(token)
                    && !stem.contains(&format!("-{token}"))
                    && !stem.contains(&format!("{token}_")),
                "garbage filename segment '{token}' in {}",
                path.display()
            );
        }
    }
    Ok(())
}
