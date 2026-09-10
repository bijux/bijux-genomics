use super::{build_fastq_container_reference_rows, build_fastq_environment_rows};
use crate::domain::{BindingSpec, LoadedSpecs, SourceSpec};
use crate::io::compilation::load_compilation_evidence;
use anyhow::Result;
use std::path::Path;

fn authored_bindings(root: &Path) -> Result<LoadedSpecs> {
    let mut loaded = LoadedSpecs::default();
    std::fs::create_dir_all(root.join("stages"))?;
    std::fs::write(root.join("execution.yaml"), "stages:\n  - stage_id: fastq.trim_reads\n    execution_status: governed\n    runtime_support: supported\n    normalization_support: supported\n    benchmark_support: supported\n    default_tool: fastp\n    admitted_tools: [fastp]\n")?;
    std::fs::write(
        root.join("registry.toml"),
        "[[tools]]\nid = \"fastp\"\ncontainer_ref = \"docker://first\"\n",
    )?;
    for (id, kind, locator) in [
        ("source.fastq.execution-support", "repo_file", "execution.yaml"),
        ("source.fastq.stage-contracts", "repo_directory", "stages"),
        ("source.fastq.tool-registry", "repo_file", "registry.toml"),
    ] {
        let source: SourceSpec = serde_json::from_value(serde_json::json!({
            "schema_version": "bijux.science.source.v1", "source_id": id,
            "kind": kind, "title": id, "locator": locator, "authority": "governed"
        }))?;
        loaded.sources.insert(id.to_string(), source);
    }
    let binding: BindingSpec = serde_json::from_value(serde_json::json!({
        "schema_version": "bijux.science.binding.v1",
        "binding_id": "binding.fastq.environment", "decision_id": "decision.fastq.environment",
        "target_type": "fastq_stage_tool_environment_matrix", "target_ref": "fastq",
        "enforcement_level": "required", "target_domain": "fastq",
        "source_ids": ["source.fastq.execution-support", "source.fastq.stage-contracts", "source.fastq.tool-registry"]
    }))?;
    loaded.bindings.insert(binding.binding_id.to_string(), binding);
    Ok(loaded)
}

#[test]
fn admitted_evidence_keeps_environment_and_container_views_consistent_across_source_changes(
) -> Result<()> {
    let scratch = bijux_dna_infra::temp_dir("science-compilation-")?;
    let loaded = authored_bindings(scratch.path())?;
    let first = load_compilation_evidence(scratch.path(), &loaded)?;
    let environment = build_fastq_environment_rows(&loaded, &first);
    std::fs::write(
        scratch.path().join("registry.toml"),
        "[[tools]]\nid = \"fastp\"\ncontainer_ref = \"docker://second\"\n",
    )?;
    let second = load_compilation_evidence(scratch.path(), &loaded)?;
    let fresh_environment = build_fastq_environment_rows(&loaded, &second);
    scratch.close()?;
    assert_eq!(environment.len(), 1);
    assert_eq!(environment[0].container_ref, "docker://first");
    assert_eq!(fresh_environment[0].container_ref, "docker://second");
    let containers = build_fastq_container_reference_rows(&first, &environment);
    assert_eq!(containers[0].container_ref, environment[0].container_ref);
    assert_eq!(build_fastq_environment_rows(&loaded, &first), environment);
    Ok(())
}

#[test]
fn a_shared_source_read_does_not_admit_a_binding_missing_its_source_relationship() -> Result<()> {
    let scratch = bijux_dna_infra::temp_dir("science-binding-")?;
    let mut loaded = authored_bindings(scratch.path())?;
    let mut invalid = loaded
        .bindings
        .values()
        .next()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("fixture binding"))?;
    invalid.binding_id =
        crate::domain::BindingId::parse("binding.fastq.unbound").map_err(anyhow::Error::msg)?;
    invalid.source_ids.retain(|id| id.as_str() != "source.fastq.tool-registry");
    loaded.bindings.insert(invalid.binding_id.to_string(), invalid);
    let error = load_compilation_evidence(scratch.path(), &loaded)
        .err()
        .ok_or_else(|| anyhow::anyhow!("missing binding source was admitted"))?;
    assert!(error
        .to_string()
        .contains("binding is missing required source source.fastq.tool-registry"));
    Ok(())
}
