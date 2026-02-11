use std::collections::{BTreeMap, BTreeSet};

use bijux_dna_core::ids::StageId;
use bijux_dna_domain_bam::BamStage;
use bijux_dna_domain_fastq::stage_param_descriptor;
use bijux_dna_pipelines::registry::{bam_profiles, cross_profiles, fastq_profiles};
use bijux_dna_pipelines::DefaultParams;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct StagesConfig {
    stages: Vec<StageEntry>,
}

#[derive(Debug, Deserialize)]
struct StageEntry {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ParamRegistryFile {
    entries: Vec<ParamRegistryEntry>,
}

#[derive(Debug, Deserialize)]
struct ParamRegistryEntry {
    stage_id: String,
    param_type_id: String,
    schema_version: String,
}

fn expected_registry(stage_id: &str) -> (String, String) {
    let stage = StageId::new(stage_id.to_string());
    if stage_id.starts_with("fastq.") {
        if let Some(descriptor) = stage_param_descriptor(&stage) {
            return (
                descriptor.param_type_id.to_string(),
                descriptor.schema_version.to_string(),
            );
        }
        return ("paramless".to_string(), "none".to_string());
    }
    if stage_id.starts_with("bam.") {
        if BamStage::try_from(stage_id).is_ok() {
            return (stage_id.to_string(), "bijux.bam.params.v1".to_string());
        }
        return ("paramless".to_string(), "none".to_string());
    }
    ("paramless".to_string(), "none".to_string())
}

fn profile_param_kind(params: &DefaultParams) -> &'static str {
    match params {
        DefaultParams::FastqValidate(_) => "fastq.validate",
        DefaultParams::FastqStats(_) => "fastq.stats",
        DefaultParams::FastqCorrect(_) => "fastq.correct",
        DefaultParams::FastqUmi(_) => "fastq.umi",
        DefaultParams::FastqDetectAdapters(_) => "fastq.detect_adapters",
        DefaultParams::FastqTrim(_) => "fastq.trim",
        DefaultParams::FastqFilter(_) => "fastq.filter",
        DefaultParams::FastqQcPost(_) => "fastq.qc_post",
        DefaultParams::FastqPreprocess(_) => "fastq.preprocess",
        DefaultParams::FastqMerge(_) => "fastq.merge",
        DefaultParams::FastqScreen(_) => "fastq.screen",
        DefaultParams::Bam(_) => "bam.effective",
        DefaultParams::Vcf(_) => "vcf.effective",
        DefaultParams::Empty(_) => "paramless",
    }
}

fn registry_entries() -> BTreeMap<String, (String, String)> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("configs");
    let registry_text = std::fs::read_to_string(root.join("param_registry.toml"))
        .unwrap_or_else(|err| panic!("read param_registry.toml failed: {err}"));
    let registry: ParamRegistryFile = toml::from_str(&registry_text)
        .unwrap_or_else(|err| panic!("parse param_registry.toml failed: {err}"));
    registry
        .entries
        .into_iter()
        .map(|entry| (entry.stage_id, (entry.param_type_id, entry.schema_version)))
        .collect()
}

#[test]
fn param_registry_covers_all_stage_ids() {
    let entries = registry_entries();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("configs");
    let stages_text = std::fs::read_to_string(root.join("stages.toml"))
        .unwrap_or_else(|err| panic!("read stages.toml failed: {err}"));
    let stages: StagesConfig = toml::from_str(&stages_text)
        .unwrap_or_else(|err| panic!("parse stages.toml failed: {err}"));

    let stage_ids: BTreeSet<String> = stages.stages.into_iter().map(|stage| stage.id).collect();
    let registry_ids: BTreeSet<String> = entries.keys().cloned().collect();
    assert!(
        stage_ids.is_subset(&registry_ids),
        "param registry must include every stage id from configs/stages.toml",
    );
}

#[test]
fn param_registry_matches_domain_mappings() {
    let entries = registry_entries();
    for (stage_id, (param_type_id, schema_version)) in entries {
        let (expected_type, expected_schema) = expected_registry(&stage_id);
        assert_eq!(
            param_type_id, expected_type,
            "registry param type mismatch for {stage_id}",
        );
        assert_eq!(
            schema_version, expected_schema,
            "registry schema version mismatch for {stage_id}",
        );
    }
}

#[test]
fn supported_stages_do_not_use_wrong_param_kind() {
    let entries = registry_entries();
    let mut profiles = Vec::new();
    profiles.extend(fastq_profiles());
    profiles.extend(bam_profiles());
    profiles.extend(cross_profiles());

    for profile in profiles {
        for stage in &profile.capabilities.required_stages {
            let stage_id = StageId::new((*stage).to_string());
            let default_expected = expected_registry(stage);
            let (expected_kind, _schema) = entries.get(*stage).cloned().unwrap_or(default_expected);
            if expected_kind == "paramless" {
                continue;
            }
            let params = profile.defaults.params.get(&stage_id).unwrap_or_else(|| {
                panic!("profile {} missing params for stage {stage}", profile.id)
            });
            let actual_kind = profile_param_kind(params);
            if stage.starts_with("bam.") {
                assert!(
                    matches!(params, DefaultParams::Bam(_)),
                    "profile {} stage {} must use BAM param kind, got {}",
                    profile.id,
                    stage,
                    actual_kind
                );
            } else {
                assert_eq!(
                    actual_kind, expected_kind,
                    "profile {} stage {} uses wrong param kind",
                    profile.id, stage
                );
            }
        }
    }
}
