use std::path::PathBuf;

use anyhow::Result;

fn v1_surface() -> Result<String> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("v1");
    let mut contents = String::new();
    for module in ["plan.rs", "run.rs", "report.rs", "bench.rs"] {
        contents.push_str(&std::fs::read_to_string(base.join(module))?);
        contents.push('\n');
    }
    Ok(contents)
}

#[test]
fn public_types_are_documented_and_v1_scoped() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let v1_surface = v1_surface()?;
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        let content = std::fs::read_to_string(entry.path())?;
        let mut lines: Vec<&str> = content.lines().collect();
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("pub struct ")
                || trimmed.starts_with("pub enum ")
                || trimmed.starts_with("pub trait ")
                || trimmed.starts_with("pub type ")
            {
                let name = trimmed
                    .split_whitespace()
                    .nth(2)
                    .unwrap_or("")
                    .trim_end_matches('{')
                    .trim_end_matches(';');
                let mut doc_block = Vec::new();
                let mut i = idx;
                while i > 0 {
                    i -= 1;
                    let doc = lines[i].trim_start();
                    if doc.starts_with("///") {
                        doc_block.push(doc.to_string());
                    } else if doc.is_empty() {
                        continue;
                    } else {
                        break;
                    }
                }
                let has_stability = doc_block.iter().any(|doc| doc.contains("Stability:"));
                if !has_stability || !v1_surface.contains(name) {
                    offenders.push(format!(
                        "{}:{} ({name})",
                        entry.path().display(),
                        idx + 1
                    ));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "public types must be documented with Stability and re-exported via v1: {offenders:?}"
    );
    Ok(())
}
