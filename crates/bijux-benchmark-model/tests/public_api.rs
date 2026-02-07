use std::fs;
use std::path::PathBuf;

#[test]
fn public_api_snapshot() -> anyhow::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let lib_path = manifest_dir.join("src").join("lib.rs");
    let raw = fs::read_to_string(&lib_path)?;
    let mut items = Vec::new();
    let mut skip_next_pub = false;
    let mut in_pub_block = 0usize;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#[cfg(test)]") {
            skip_next_pub = true;
            continue;
        }
        if in_pub_block > 0 {
            if trimmed.contains('{') {
                in_pub_block += trimmed.matches('{').count();
            }
            if trimmed.contains('}') {
                let closes = trimmed.matches('}').count();
                in_pub_block = in_pub_block.saturating_sub(closes);
            }
            continue;
        }
        if skip_next_pub {
            if trimmed.starts_with("pub ") {
                skip_next_pub = false;
                continue;
            }
            if !trimmed.is_empty() {
                skip_next_pub = false;
            }
        }
        if trimmed.starts_with("pub ") {
            if (trimmed.starts_with("pub struct ") || trimmed.starts_with("pub enum "))
                && trimmed.contains('{')
            {
                in_pub_block = trimmed.matches('{').count();
            }
            items.push(trimmed.to_string());
        }
    }
    let rendered = items.join("\n");
    let snapshot_path = manifest_dir
        .join("tests")
        .join("snapshots")
        .join("public_api.txt");
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}
