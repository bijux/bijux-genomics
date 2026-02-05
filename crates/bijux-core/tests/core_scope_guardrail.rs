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
fn core_scope_guardrail() -> Result<(), Box<dyn std::error::Error>> {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = crate_root.join("src");
    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files)?;

    let forbidden = [
        "telemetry",
        "observability",
        "opentelemetry",
        "tracing::",
        "tokio::",
        "std::process::Command",
        "reqwest",
        "sqlx",
        "hyper",
        "actix",
        "warp",
        "rocket",
    ];

    let mut violations = Vec::new();
    for file in files {
        let content = fs::read_to_string(&file)?;
        for token in &forbidden {
            if content.contains(token) {
                violations.push(format!(
                    "{} contains forbidden token: {}",
                    file.display(),
                    token
                ));
            }
        }
    }

    if violations.is_empty() {
        return Ok(());
    }

    Err(format!("Core scope violations:\n{}", violations.join("\n")).into())
}
