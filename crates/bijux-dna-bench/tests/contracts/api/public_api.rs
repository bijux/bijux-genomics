use std::fs;

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-dna-bench__{group}__{name}")
}

#[test]
fn public_api_snapshot() -> anyhow::Result<()> {
    let manifest_dir = crate::support::crate_root("bijux-dna-bench")?;
    let lib_path = manifest_dir.join("src").join("lib.rs");
    let raw = fs::read_to_string(&lib_path)?;
    let mut items = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub ") {
            items.push(trimmed.to_string());
        }
    }
    let rendered = items.join("\n");
    let snapshot_file = format!("{}.txt", snapshot_name("schemas", "public_api"));
    let snapshot_path = manifest_dir.join("tests").join("snapshots").join(snapshot_file);
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}
