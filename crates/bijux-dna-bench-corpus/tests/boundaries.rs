//! Keep local corpus admission independently compilable from application orchestration.
mod support;
use anyhow::{ensure, Result};
use std::collections::BTreeSet;

fn admit_dependencies(manifest: &toml::Value) -> Result<()> {
    let allowed = BTreeSet::from([
        "anyhow",
        "schemars",
        "chrono",
        "serde",
        "serde_json",
        "toml",
        "flate2",
        "bijux-dna-domain-bam",
        "bijux-dna-domain-vcf",
        "bijux-dna-db-ref",
        "bijux-dna-infra",
        "bijux-dna-core",
    ]);
    let dependencies = manifest["dependencies"]
        .as_table()
        .ok_or_else(|| anyhow::anyhow!("missing dependencies"))?;
    for (name, declaration) in dependencies {
        ensure!(allowed.contains(name.as_str()), "corpus admission cannot depend on {name}");
        if name.starts_with("bijux-dna-") || name == "schemars" || name == "chrono" {
            ensure!(
                declaration.get("optional").and_then(toml::Value::as_bool) == Some(true),
                "specialized dependency {name} must remain optional"
            );
        }
    }
    if dependencies.contains_key("schemars") {
        let source = manifest["features"]["source-contract"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("missing source-contract feature"))?;
        ensure!(
            source == &[toml::Value::String("dep:schemars".into())],
            "source contracts must only enable schema support"
        );
        if let Some(normalized) = manifest["features"].get("normalized") {
            ensure!(
                normalized.as_array()
                    == Some(&vec![
                        toml::Value::String("source-contract".into()),
                        toml::Value::String("dep:bijux-dna-infra".into())
                    ]),
                "normalized admission must only enable source contracts and infrastructure"
            );
        }
    }
    if dependencies.contains_key("chrono") {
        ensure!(
            manifest["features"]["run-contract"].as_array()
                == Some(&vec![
                    toml::Value::String("dep:schemars".into()),
                    toml::Value::String("dep:chrono".into())
                ]),
            "run contracts must only enable schema and timestamp support"
        );
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

#[test]
fn source_features_reject_implicit_schema_and_native_engine_activation() -> Result<()> {
    let baseline: toml::Value = toml::from_str(
        r#"
[dependencies]
schemars = { version = "1", optional = true }
bijux-dna-infra = { version = "1", optional = true }
[features]
default = []
source-contract = ["dep:schemars"]
normalized = ["source-contract", "dep:bijux-dna-infra"]
"#,
    )?;
    admit_dependencies(&baseline)?;
    let mut changed = baseline.clone();
    changed["dependencies"]["schemars"]["optional"] = toml::Value::Boolean(false);
    assert!(admit_dependencies(&changed).is_err());
    for feature in ["source-contract", "normalized"] {
        let mut changed = baseline.clone();
        changed["features"][feature]
            .as_array_mut()
            .ok_or_else(|| anyhow::anyhow!("feature array"))?
            .push(toml::Value::String("alignment".into()));
        assert!(admit_dependencies(&changed).is_err(), "{feature}");
    }
    Ok(())
}

#[test]
fn run_contract_rejects_implicit_time_and_native_engine_dependencies() -> Result<()> {
    let baseline: toml::Value = toml::from_str(
        r#"
[dependencies]
chrono = { version = "1", optional = true }
schemars = { version = "1", optional = true }
[features]
default = []
source-contract = ["dep:schemars"]
run-contract = ["dep:schemars", "dep:chrono"]
"#,
    )?;
    admit_dependencies(&baseline)?;
    let mut changed = baseline.clone();
    changed["dependencies"]["chrono"]["optional"] = toml::Value::Boolean(false);
    assert!(admit_dependencies(&changed).is_err());
    let mut changed = baseline;
    changed["features"]["run-contract"]
        .as_array_mut()
        .ok_or_else(|| anyhow::anyhow!("feature array"))?
        .push(toml::Value::String("alignment".into()));
    assert!(admit_dependencies(&changed).is_err());
    Ok(())
}
