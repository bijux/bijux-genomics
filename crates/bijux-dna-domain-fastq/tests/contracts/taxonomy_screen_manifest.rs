use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("workspace root")
}

fn parse_yaml(path: &Path) -> Result<Value> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    bijux_dna_infra::formats::parse_yaml(&raw).with_context(|| format!("parse {}", path.display()))
}

#[test]
fn taxonomy_screen_manifest_keeps_read_classifier_tools_only() -> Result<()> {
    let path = workspace_root()?.join("domain/fastq/stages/screen_taxonomy.yaml");
    let yaml = parse_yaml(&path)?;
    let compatible_tools = yaml
        .get("compatible_tools")
        .and_then(Value::as_array)
        .context("compatible_tools")?
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();

    assert!(
        !compatible_tools.contains(&"metaphlan"),
        "fastq.screen_taxonomy must not admit marker-profile tools under the read-classifier contract"
    );
    assert!(
        !compatible_tools.contains(&"fastq_screen"),
        "fastq.screen_taxonomy must not admit mapping-QC tools under the read-classifier contract"
    );
    Ok(())
}

#[test]
fn taxonomy_screen_manifest_publishes_governed_taxonomy_metrics() -> Result<()> {
    let path = workspace_root()?.join("domain/fastq/stages/screen_taxonomy.yaml");
    let yaml = parse_yaml(&path)?;
    let metrics = yaml
        .get("metrics")
        .and_then(Value::as_array)
        .context("metrics")?
        .iter()
        .filter_map(|metric| metric.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>();

    assert!(
        metrics.contains(&"classified_fraction"),
        "fastq.screen_taxonomy must publish classified_fraction in the governed stage contract"
    );
    assert!(
        metrics.contains(&"unclassified_fraction"),
        "fastq.screen_taxonomy must publish unclassified_fraction in the governed stage contract"
    );
    assert!(
        metrics.contains(&"top_taxa"),
        "fastq.screen_taxonomy must publish top_taxa in the governed stage contract"
    );
    Ok(())
}

#[test]
fn taxonomy_screen_tool_manifests_admit_optional_mate_inputs() -> Result<()> {
    for tool in ["kraken2", "centrifuge", "kaiju"] {
        let path = workspace_root()?.join(format!("domain/fastq/tools/{tool}.yaml"));
        let yaml = parse_yaml(&path)?;
        let optional_inputs = yaml
            .get("execution_contract")
            .and_then(|value| value.get("optional_inputs"))
            .and_then(Value::as_array)
            .context("execution_contract.optional_inputs")?
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>();
        assert!(
            optional_inputs.contains(&"reads_r2"),
            "{tool} must admit reads_r2 as an optional governed screen-taxonomy input"
        );
    }
    Ok(())
}

#[test]
fn taxonomy_screen_defaults_stay_screening_only_without_truth_conditions() {
    let defaults = bijux_dna_domain_fastq::params::defaults::screen_defaults(true);
    assert_eq!(
        defaults.interpretation_boundary,
        bijux_dna_domain_fastq::params::screen::TaxonomyInterpretationBoundary::ScreeningOnly
    );
    assert!(
        defaults.truth_conditions.is_empty(),
        "screen-taxonomy defaults must not silently imply definitive classification"
    );
}
