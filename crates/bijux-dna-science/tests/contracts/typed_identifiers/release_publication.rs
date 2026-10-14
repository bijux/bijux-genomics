use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_science::compile::{compile_workspace, load_specs};
use bijux_dna_science::domain::ScienceReleaseId;
use bijux_dna_science::release::cut_release;
use serde_json::{json, Value};

const VALID_RELEASE: &str = "release.private-control";
const ESCAPING_RELEASE: &str = "release.controlled/../../../escaped";

fn write_spec(root: &Path, relative: &str, value: &Value) -> Result<()> {
    // JSON documents are valid YAML and exercise the maintained authored YAML loader.
    bijux_dna_infra::write_string(root.join(relative), &serde_json::to_string_pretty(value)?)?;
    Ok(())
}

fn write_release(root: &Path, release_id: &str) -> Result<()> {
    write_spec(
        root,
        "science/specs/releases/manifests/private.yaml",
        &json!({
            "schema_version": "bijux.science.release.v1", "release_id": release_id,
            "title": "Private release boundary", "status": "candidate",
            "binding_ids": ["binding.fastq.environment"], "claim_ids": ["claim.fastq.context"]
        }),
    )
}

fn authored_workspace(root: &Path) -> Result<()> {
    bijux_dna_infra::ensure_dir(root.join("stages"))?;
    bijux_dna_infra::write_string(root.join("execution.yaml"), "stages:\n  - stage_id: fastq.trim_reads\n    execution_status: governed\n    runtime_support: supported\n    normalization_support: supported\n    benchmark_support: supported\n    default_tool: fastp\n    admitted_tools: [fastp]\n")?;
    bijux_dna_infra::write_string(
        root.join("registry.toml"),
        "[[tools]]\nid = \"fastp\"\ncontainer_ref = \"docker://private\"\n",
    )?;
    for (name, kind, locator) in [
        ("execution-support", "repo_file", "execution.yaml"),
        ("stage-contracts", "repo_directory", "stages"),
        ("tool-registry", "repo_file", "registry.toml"),
    ] {
        write_spec(
            root,
            &format!("science/specs/evidence/sources/{name}.yaml"),
            &json!({
                "schema_version": "bijux.science.source.v1", "source_id": format!("source.fastq.{name}"),
                "kind": kind, "title": name, "locator": locator, "authority": "private-contract"
            }),
        )?;
    }
    write_spec(
        root,
        "science/specs/evidence/evidences/context.yaml",
        &json!({
            "schema_version": "bijux.science.evidence.v1", "evidence_id": "evidence.fastq.context",
            "statement": "Private contract source", "status": "accepted", "strength": "governing"
        }),
    )?;
    write_spec(
        root,
        "science/specs/evidence/claims/context.yaml",
        &json!({
            "schema_version": "bijux.science.claim.v1", "claim_id": "claim.fastq.context",
            "statement": "Private fixture supports boundary checks", "scope": "fixture", "subject": "fixture",
            "predicate": "supports", "object": "boundary", "status": "accepted", "owner": "science",
            "review_due": "2026-01-01", "supports": ["evidence.fastq.context"]
        }),
    )?;
    write_spec(
        root,
        "science/specs/evidence/reasoning/context.yaml",
        &json!({
            "schema_version": "bijux.science.reasoning.v1", "reasoning_id": "reasoning.fastq.context",
            "claim_ids": ["claim.fastq.context"], "residual_risk": "Fixture scope only", "conclusion": "Ready for boundary checks"
        }),
    )?;
    write_spec(
        root,
        "science/specs/evidence/decisions/context.yaml",
        &json!({
            "schema_version": "bijux.science.decision.v1", "decision_id": "decision.fastq.context",
            "statement": "Use fixture for boundary checks", "decision_type": "fixture_support", "status": "accepted",
            "reasoning": "reasoning.fastq.context", "owner": "science", "effective_from": "2026-01-01", "confidence": "high"
        }),
    )?;
    write_spec(
        root,
        "science/specs/evidence/bindings/context.yaml",
        &json!({
            "schema_version": "bijux.science.binding.v1", "binding_id": "binding.fastq.environment",
            "decision_id": "decision.fastq.context", "target_type": "fastq_stage_tool_environment_matrix",
            "target_ref": "fastq", "enforcement_level": "required", "target_domain": "fastq",
            "source_ids": ["source.fastq.execution-support", "source.fastq.stage-contracts", "source.fastq.tool-registry"]
        }),
    )?;
    write_release(root, VALID_RELEASE)?;
    Ok(())
}

fn tree_bytes(root: &Path) -> Result<BTreeMap<PathBuf, Option<Vec<u8>>>> {
    let mut entries = BTreeMap::new();
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry?;
        assert!(!entry.file_type().is_symlink(), "private input must not redirect");
        let relative = entry.path().strip_prefix(root)?.to_path_buf();
        let contents =
            if entry.file_type().is_file() { Some(std::fs::read(entry.path())?) } else { None };
        entries.insert(relative, contents);
    }
    Ok(entries)
}

#[test]
fn admitted_release_fixture_publishes_under_its_owned_identity() -> Result<()> {
    let owner = bijux_dna_infra::temp_dir("science-release-contract-")?;
    authored_workspace(owner.path())?;
    let loaded = load_specs(owner.path())?;
    assert_eq!(loaded.releases[VALID_RELEASE].release_id.as_str(), VALID_RELEASE);
    let compiled = compile_workspace(owner.path())?;
    assert_eq!(compiled.index.releases, 1);
    assert_eq!(compiled.fastq_environment_rows.len(), 1);
    cut_release(owner.path(), VALID_RELEASE)?;
    let release_root = owner.path().join("artifacts/science-releases").join(VALID_RELEASE);
    let report: Value =
        serde_json::from_slice(&std::fs::read(release_root.join("indexes/release.json"))?)?;
    assert_eq!(report["release_id"], VALID_RELEASE);
    assert!(release_root.join("evidence/source_inventory.tsv").is_file());
    assert!(!owner.path().join("escaped").exists());
    Ok(())
}

#[test]
fn authored_release_loader_refuses_identifier_grammar_before_publication() -> Result<()> {
    let owner = bijux_dna_infra::temp_dir("science-release-loader-")?;
    authored_workspace(owner.path())?;
    compile_workspace(owner.path())?;
    write_release(owner.path(), ESCAPING_RELEASE)?;
    let before = tree_bytes(owner.path())?;
    let result = load_specs(owner.path());
    assert!(result.is_err(), "authored loader admitted traversal release ID {ESCAPING_RELEASE:?}");
    let error = result.err().context("invalid release ID unexpectedly loaded")?;
    let grammar = ScienceReleaseId::parse(ESCAPING_RELEASE)
        .err()
        .context("fixture must violate maintained grammar")?;
    let diagnostic = format!("{error:#}");
    assert!(diagnostic.contains(&grammar), "actual loader refusal: {diagnostic}");
    assert!(diagnostic.contains("private.yaml"), "actual loader context: {diagnostic}");
    assert_eq!(tree_bytes(owner.path())?, before);
    Ok(())
}

#[test]
fn release_cut_refuses_registered_traversal_before_owned_tree_changes() -> Result<()> {
    let owner = bijux_dna_infra::temp_dir("science-release-refusal-")?;
    authored_workspace(owner.path())?;
    compile_workspace(owner.path())?;
    write_release(owner.path(), ESCAPING_RELEASE)?;
    // Every traversal component remains inside this private fixture; no shared path is probed.
    bijux_dna_infra::ensure_dir(
        owner.path().join("artifacts/science-releases/release.controlled"),
    )?;
    bijux_dna_infra::write_string(owner.path().join("retained-marker"), "preserve private input")?;
    let before = tree_bytes(owner.path())?;
    let result = cut_release(owner.path(), ESCAPING_RELEASE);
    assert!(
        result.is_err(),
        "registered traversal was published; private escaped output exists={}",
        owner.path().join("escaped/indexes/release.json").is_file()
    );
    let error = result.err().ok_or_else(|| anyhow!("invalid release was published"))?;
    let grammar = ScienceReleaseId::parse(ESCAPING_RELEASE)
        .err()
        .context("fixture must violate maintained grammar")?;
    let diagnostic = format!("{error:#}");
    assert!(diagnostic.contains(&grammar), "actual cut refusal: {diagnostic}");
    assert_eq!(tree_bytes(owner.path())?, before, "refusal must precede all publication effects");
    Ok(())
}

#[test]
fn unmatched_release_request_keeps_manifest_lookup_error_priority() -> Result<()> {
    let owner = bijux_dna_infra::temp_dir("science-release-missing-")?;
    authored_workspace(owner.path())?;
    let before = tree_bytes(owner.path())?;
    let missing = "../unregistered";
    let error = cut_release(owner.path(), missing)
        .err()
        .context("missing release unexpectedly published")?;
    assert_eq!(error.to_string(), format!("release manifest not found for {missing}"));
    assert_eq!(tree_bytes(owner.path())?, before);
    Ok(())
}
