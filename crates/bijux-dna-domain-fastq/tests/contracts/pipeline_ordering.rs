use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_yaml::Value;

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("workspace root")
}

fn parse_yaml(path: &Path) -> Result<Value> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

#[test]
fn pre_hpc_pipeline_depletes_before_merging_pairs() -> Result<()> {
    let index = parse_yaml(&workspace_root()?.join("domain/fastq/index.yaml"))?;
    let pipeline = index
        .get("pipeline_compositions")
        .and_then(Value::as_mapping)
        .and_then(|compositions| compositions.get(Value::String("pre_hpc_best".to_string())))
        .and_then(Value::as_sequence)
        .context("pipeline_compositions.pre_hpc_best")?
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();

    let merge_position = pipeline
        .iter()
        .position(|stage_id| *stage_id == "fastq.merge_pairs")
        .context("fastq.merge_pairs missing from pre_hpc_best")?;
    let host_position = pipeline
        .iter()
        .position(|stage_id| *stage_id == "fastq.deplete_host")
        .context("fastq.deplete_host missing from pre_hpc_best")?;
    let contaminant_position = pipeline
        .iter()
        .position(|stage_id| *stage_id == "fastq.deplete_reference_contaminants")
        .context("fastq.deplete_reference_contaminants missing from pre_hpc_best")?;

    assert!(
        host_position < merge_position,
        "fastq.deplete_host must run before fastq.merge_pairs in pre_hpc_best"
    );
    assert!(
        contaminant_position < merge_position,
        "fastq.deplete_reference_contaminants must run before fastq.merge_pairs in pre_hpc_best"
    );
    Ok(())
}

#[test]
fn canonical_shotgun_order_covers_reference_and_reporting_stages() {
    let stages = bijux_dna_domain_fastq::canonical_stage_order()
        .into_iter()
        .map(|stage| stage.to_string())
        .collect::<Vec<_>>();

    for required in [
        "fastq.index_reference",
        "fastq.deplete_host",
        "fastq.deplete_reference_contaminants",
        "fastq.profile_overrepresented_sequences",
        "fastq.screen_taxonomy",
        "fastq.report_qc",
    ] {
        assert!(
            stages.iter().any(|stage| stage == required),
            "canonical FASTQ order missing stage {required}"
        );
    }
}

#[test]
fn canonical_amplicon_order_uses_supported_feature_stage() {
    let stages = bijux_dna_domain_fastq::canonical_amplicon_stage_order()
        .into_iter()
        .map(|stage| stage.to_string())
        .collect::<Vec<_>>();

    assert!(
        stages.iter().any(|stage| stage == "fastq.cluster_otus"),
        "canonical amplicon order must include clustering"
    );
    assert!(
        !stages.iter().any(|stage| stage == "fastq.infer_asvs"),
        "canonical amplicon order must not default to planned ASV inference"
    );
}

#[test]
fn closed_screen_taxonomy_is_not_marked_experimental() {
    let stage_id = bijux_dna_core::ids::StageId::from_static("fastq.screen_taxonomy");
    assert_eq!(
        bijux_dna_domain_fastq::stage_criticality(&stage_id),
        Some(bijux_dna_domain_fastq::StageCriticality::Optional),
        "closed FASTQ taxonomy screening must not be labeled experimental"
    );
}
