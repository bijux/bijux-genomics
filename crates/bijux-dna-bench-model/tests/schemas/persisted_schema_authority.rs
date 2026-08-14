use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use bijux_dna_bench_model::persisted_benchmark_schemas;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn checked_in_persisted_schemas_match_the_model_authority() -> Result<()> {
    let root = repository_root();
    let mut paths = BTreeSet::new();

    for schema in persisted_benchmark_schemas() {
        assert!(
            paths.insert(schema.repository_path),
            "duplicate persisted benchmark schema path: {}",
            schema.repository_path
        );
        let checked_in_path = root.join(schema.repository_path);
        let generated = serde_json::to_value(schema.render())?;
        if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
            let mut bytes = serde_json::to_vec_pretty(&generated)?;
            bytes.push(b'\n');
            fs::write(&checked_in_path, bytes)
                .with_context(|| format!("write {}", checked_in_path.display()))?;
        }
        let checked_in: serde_json::Value = serde_json::from_slice(
            &fs::read(&checked_in_path)
                .with_context(|| format!("read {}", checked_in_path.display()))?,
        )
        .with_context(|| format!("parse {}", checked_in_path.display()))?;
        assert_eq!(
            checked_in, generated,
            "persisted benchmark schema drifted: {}",
            schema.repository_path
        );
    }

    assert_eq!(paths.len(), persisted_benchmark_schemas().len());
    Ok(())
}
