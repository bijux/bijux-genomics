//! Keep local corpus admission independently compilable from application orchestration.
mod support;
use anyhow::{ensure, Result};
use std::collections::BTreeSet;

fn admit_dependencies(manifest: &toml::Value) -> Result<()> {
    let allowed = BTreeSet::from([
        "anyhow",
        "serde",
        "serde_json",
        "toml",
        "flate2",
        "bijux-dna-domain-bam",
        "bijux-dna-domain-vcf",
    ]);
    let dependencies = manifest["dependencies"]
        .as_table()
        .ok_or_else(|| anyhow::anyhow!("missing dependencies"))?;
    for (name, declaration) in dependencies {
        ensure!(allowed.contains(name.as_str()), "corpus admission cannot depend on {name}");
        if name.starts_with("bijux-dna-domain-") {
            ensure!(
                declaration.get("optional").and_then(toml::Value::as_bool) == Some(true),
                "native domain {name} must remain optional"
            );
        }
    }
    let default = manifest["features"]["default"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("missing default feature set"))?;
    ensure!(
        default.is_empty(),
        "default corpus admission must not activate native domain families"
    );
    Ok(())
}

#[test]
fn corpus_manifest_preserves_independent_compilation() -> Result<()> {
    let path = support::repo_root().join("crates/bijux-dna-bench-corpus/Cargo.toml");
    let manifest = toml::from_str(&std::fs::read_to_string(path)?)?;
    admit_dependencies(&manifest)
}

#[test]
fn admission_rejects_application_back_edges_and_implicit_native_domains() -> Result<()> {
    let baseline: toml::Value = toml::from_str(
        r#"
[dependencies]
serde = "1"
bijux-dna-domain-bam = { version = "1", optional = true }
[features]
default = []
"#,
    )?;
    admit_dependencies(&baseline)?;
    for forbidden in [
        "bijux-dna",
        "bijux-dna-api",
        "bijux-dna-bench",
        "bijux-dna-bench-model",
        "bijux-dna-runner",
        "bijux-dna-stages-vcf",
    ] {
        let mut changed = baseline.clone();
        changed["dependencies"]
            .as_table_mut()
            .ok_or_else(|| anyhow::anyhow!("missing table"))?
            .insert(forbidden.into(), toml::Value::String("1".into()));
        assert!(admit_dependencies(&changed).is_err(), "{forbidden}");
    }
    let mut changed = baseline.clone();
    changed["dependencies"]["bijux-dna-domain-bam"]["optional"] = toml::Value::Boolean(false);
    assert!(admit_dependencies(&changed).is_err());
    let mut changed = baseline;
    changed["features"]["default"] =
        toml::Value::Array(vec![toml::Value::String("alignment".into())]);
    assert!(admit_dependencies(&changed).is_err());
    Ok(())
}
