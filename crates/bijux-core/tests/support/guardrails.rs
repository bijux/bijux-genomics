use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
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

#[allow(dead_code)]
pub fn assert_module_depth(src_dir: &Path) -> io::Result<()> {
    let mut files = Vec::new();
    collect_rs_files(src_dir, &mut files)?;
    for path in files {
        let rel = path.strip_prefix(src_dir).unwrap_or(&path);
        let components: Vec<_> = rel.components().collect();
        if components.len() <= 3 {
            continue;
        }
        let is_mod_rs = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "mod.rs");
        if components.len() == 4 && is_mod_rs {
            continue;
        }
        panic!(
            "module depth exceeds allowed rule (src/a/b/c.rs or mod.rs at each level): {}",
            path.display()
        );
    }
    Ok(())
}

#[allow(dead_code)]
fn line_count(path: &Path) -> io::Result<usize> {
    let content = fs::read_to_string(path)?;
    Ok(content.lines().count())
}

#[allow(dead_code)]
pub fn assert_loc_budget(src_dir: &Path, strict: bool) -> io::Result<()> {
    let mut files = Vec::new();
    collect_rs_files(src_dir, &mut files)?;
    let hard_limit = 1000usize;
    let soft_limit = 500usize;
    for path in files {
        let lines = line_count(&path)?;
        assert!(
            lines <= hard_limit,
            "{} has {} lines (max {})",
            path.display(),
            lines,
            hard_limit
        );
        assert!(
            !(strict && lines > soft_limit),
            "{} has {} lines (strict max {})",
            path.display(),
            lines,
            soft_limit
        );
    }
    Ok(())
}
